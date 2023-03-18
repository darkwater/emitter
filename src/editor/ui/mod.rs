use std::any::TypeId;

use bevy::{
    asset::{HandleId, ReflectAsset},
    prelude::*,
    reflect::TypeRegistryInternal,
    render::camera::{CameraProjection, Viewport},
};
use bevy_egui::EguiSet;
use bevy_inspector_egui::{
    bevy_egui::EguiContext,
    bevy_inspector::{
        self,
        hierarchy::{hierarchy_ui, SelectedEntities},
        ui_for_entities_shared_components, ui_for_entity_with_children,
    },
};
use bevy_rapier3d::{
    prelude::{Collider, CollisionGroups, RigidBody},
    render::DebugRenderContext,
};
use egui_dock::{NodeIndex, Tree};
use egui_gizmo::{GizmoMode, GizmoVisuals};
use heck::ToTitleCase;

use super::{
    mesh::{DeleteConnectedLines, MeshLine, MeshPoint, Solidify},
    EditorCamera, EditorWindow,
};
use crate::{
    collision_groups,
    line_material::{LineList, LineMaterial},
};

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            show_ui_system
                .in_base_set(CoreSet::PostUpdate)
                .before(EguiSet::ProcessOutput)
                .before(bevy::transform::TransformSystem::TransformPropagate),
        )
        .add_system(
            set_camera_viewport
                .in_base_set(CoreSet::PostUpdate)
                .after(show_ui_system),
        );
    }
}

#[derive(Debug)]
pub enum EguiWindow {
    GameView,
    Hierarchy,
    Resources,
    Assets,
    Inspector,
    MapTools,
    Options,
}

#[derive(Eq, PartialEq)]
pub enum InspectorSelection {
    None,
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, HandleId),
}

fn show_ui_system(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<EditorWindow>>()
        .get_single(world) else { return };
    let mut egui_context = egui_context.clone();

    world.resource_scope::<UiState, _>(|world, mut ui_state| {
        egui::TopBottomPanel::top("menu_bar").show(egui_context.get_mut(), |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Scene", |ui| ui.button("Save"));
            });
        });

        ui_state.ui(world, egui_context.get_mut())
    });
}

fn set_camera_viewport(
    ui_state: Res<UiState>,
    primary_window: Query<&mut Window, With<EditorWindow>>,
    egui_settings: Res<bevy_egui::EguiSettings>,
    mut cameras: Query<&mut Camera, With<EditorCamera>>,
) {
    let mut cam = cameras.single_mut();

    let Ok(window) = primary_window.get_single() else { return };

    let scale_factor = window.scale_factor() * egui_settings.scale_factor;

    let viewport_pos = ui_state.viewport_rect.left_top().to_vec2() * scale_factor as f32;
    let viewport_size = ui_state.viewport_rect.size() * scale_factor as f32;

    cam.viewport = Some(Viewport {
        physical_position: UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32),
        physical_size: UVec2::new(viewport_size.x as u32, viewport_size.y as u32),
        depth: 0.0..1.0,
    });
}

#[derive(Resource)]
pub struct UiState {
    pub tree: Tree<EguiWindow>,
    pub viewport_rect: egui::Rect,
    pub selected_entities: SelectedEntities,
    pub selection: InspectorSelection,
    pub gizmo_mode: GizmoMode,
    pub hovering_camera: bool,
}

impl UiState {
    pub fn new() -> Self {
        let mut tree = Tree::new(vec![EguiWindow::GameView]);

        let [camera, right_panel] =
            tree.split_right(NodeIndex::root(), 0.75, vec![EguiWindow::Options]);

        let [_right_top, _right_bottom] =
            tree.split_below(right_panel, 0.5, vec![EguiWindow::Inspector]);

        let [camera, _left_panel] =
            tree.split_left(camera, 0.2, vec![EguiWindow::MapTools, EguiWindow::Hierarchy]);

        let [_camera, _bottom_panel] =
            tree.split_below(camera, 0.8, vec![EguiWindow::Resources, EguiWindow::Assets]);

        Self {
            tree,
            selected_entities: SelectedEntities::default(),
            selection: InspectorSelection::None,
            viewport_rect: egui::Rect::NOTHING,
            gizmo_mode: GizmoMode::Translate,
            hovering_camera: false,
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            selected_entities: &mut self.selected_entities,
            selection: &mut self.selection,
            gizmo_mode: &mut self.gizmo_mode,
        };
        egui_dock::DockArea::new(&mut self.tree).show(ctx, &mut tab_viewer);
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

struct TabViewer<'a> {
    world: &'a mut World,
    selected_entities: &'a mut SelectedEntities,
    selection: &'a mut InspectorSelection,
    viewport_rect: &'a mut egui::Rect,
    gizmo_mode: &'a mut GizmoMode,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = EguiWindow;

    fn ui(&mut self, ui: &mut egui::Ui, window: &mut Self::Tab) {
        let type_registry = self.world.resource::<AppTypeRegistry>().0.clone();
        let type_registry = type_registry.read();

        match window {
            EguiWindow::GameView => {
                (*self.viewport_rect, _) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());

                draw_gizmo(ui, self.world, self.selected_entities, *self.gizmo_mode);
            }
            EguiWindow::Hierarchy => {
                let selected = hierarchy_ui(self.world, ui, self.selected_entities);
                if selected {
                    *self.selection = InspectorSelection::Entities;
                }
            }
            EguiWindow::Resources => select_resource(ui, &type_registry, self.selection),
            EguiWindow::Assets => select_asset(ui, &type_registry, self.world, self.selection),
            EguiWindow::Inspector => match *self.selection {
                InspectorSelection::Entities => match self.selected_entities.as_slice() {
                    &[entity] => ui_for_entity_with_children(self.world, entity, ui),
                    entities => ui_for_entities_shared_components(self.world, entities, ui),
                },
                InspectorSelection::Resource(type_id, ref name) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_resource(
                        self.world,
                        type_id,
                        ui,
                        name,
                        &type_registry,
                    )
                }
                InspectorSelection::Asset(type_id, ref name, handle) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_asset(
                        self.world,
                        type_id,
                        handle,
                        ui,
                        &type_registry,
                    );
                }
                InspectorSelection::None => {}
            },
            EguiWindow::MapTools => {
                egui::ComboBox::from_id_source("gizmo_mode")
                    .selected_text(format!("{:?}", self.gizmo_mode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(self.gizmo_mode, GizmoMode::Translate, "Translate");
                        ui.selectable_value(self.gizmo_mode, GizmoMode::Rotate, "Rotate");
                        ui.selectable_value(self.gizmo_mode, GizmoMode::Scale, "Scale");
                    });

                if ui.button("Toggle hitboxes").clicked() {
                    let mut debug_render = self.world.resource_mut::<DebugRenderContext>();
                    debug_render.enabled = !debug_render.enabled;
                }
            }
            EguiWindow::Options => {
                let mut line_query = self.world.query::<(Entity, &MeshLine)>();

                if let InspectorSelection::Entities = self.selection {
                    if ui.button("Delete entity").clicked() {
                        for entity in self.selected_entities.as_slice() {
                            self.world.despawn(*entity);
                        }
                        self.selected_entities.clear();
                    }

                    if ui.button("Delete connected lines").clicked() {
                        self.world.send_event(DeleteConnectedLines);
                    }
                } else if line_query.iter(self.world).next().is_some() {
                    let button = ui.button("Solidify mesh");
                    if button.clicked() {
                        self.world.send_event(Solidify);
                    }
                }
            }
        }
    }

    fn title(&mut self, window: &mut Self::Tab) -> egui::WidgetText {
        format!("{window:?}").to_title_case().into()
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, EguiWindow::GameView)
    }
}

fn select_resource(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistryInternal,
    selection: &mut InspectorSelection,
) {
    let mut resources: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectResource>().is_some())
        .map(|registration| (registration.short_name().to_owned(), registration.type_id()))
        .collect();
    resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

    for (resource_name, type_id) in resources {
        let selected = match *selection {
            InspectorSelection::Resource(selected, _) => selected == type_id,
            _ => false,
        };

        if ui.selectable_label(selected, &resource_name).clicked() {
            *selection = InspectorSelection::Resource(type_id, resource_name);
        }
    }
}

fn select_asset(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistryInternal,
    world: &World,
    selection: &mut InspectorSelection,
) {
    let mut assets: Vec<_> = type_registry
        .iter()
        .filter_map(|registration| {
            let reflect_asset = registration.data::<ReflectAsset>()?;
            Some((registration.short_name().to_owned(), registration.type_id(), reflect_asset))
        })
        .collect();
    assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));

    for (asset_name, asset_type_id, reflect_asset) in assets {
        let mut handles: Vec<_> = reflect_asset.ids(world).collect();
        handles.sort();

        ui.collapsing(format!("{asset_name} ({})", handles.len()), |ui| {
            for handle in handles {
                let selected = match *selection {
                    InspectorSelection::Asset(_, _, selected_id) => selected_id == handle,
                    _ => false,
                };

                if ui
                    .selectable_label(selected, format!("{:?}", handle))
                    .clicked()
                {
                    *selection =
                        InspectorSelection::Asset(asset_type_id, asset_name.clone(), handle);
                }
            }
        });
    }
}

fn draw_gizmo(
    ui: &mut egui::Ui,
    world: &mut World,
    selected_entities: &SelectedEntities,
    gizmo_mode: GizmoMode,
) {
    let (cam_transform, projection) = world
        .query_filtered::<(&GlobalTransform, &Projection), With<EditorCamera>>()
        .single(world);
    let view_matrix = Mat4::from(cam_transform.affine().inverse());
    let projection_matrix = projection.get_projection_matrix();

    for selected in selected_entities.iter() {
        let Some(transform) = world.get::<Transform>(selected) else { continue };
        let model_matrix = transform.compute_matrix();

        let Some(result) = egui_gizmo::Gizmo::new(selected)
                    .model_matrix(model_matrix.to_cols_array_2d())
                    .view_matrix(view_matrix.to_cols_array_2d())
                    .projection_matrix(projection_matrix.to_cols_array_2d())
                    .orientation(egui_gizmo::GizmoOrientation::Global)
                    .snapping(true)
                    .snap_distance(1.)
                    .mode(gizmo_mode)
                    .visuals(GizmoVisuals {
                        ..default()
                    })
                    .interact(ui)
                else { continue };

        let mut transform = world.get_mut::<Transform>(selected).unwrap();
        *transform = Transform {
            translation: Vec3::from(<[f32; 3]>::from(result.translation)),
            rotation: Quat::from_array(<[f32; 4]>::from(result.rotation)),
            scale: Vec3::from(<[f32; 3]>::from(result.scale)),
        };
    }
}
