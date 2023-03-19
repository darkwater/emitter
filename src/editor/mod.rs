use bevy::{
    core_pipeline::{
        bloom::BloomSettings, clear_color::ClearColorConfig, tonemapping::Tonemapping,
    },
    input::mouse::MouseMotion,
    math::DVec2,
    prelude::*,
    render::camera::RenderTarget,
    window::{CursorGrabMode, PresentMode, WindowMode, WindowRef, WindowResolution},
};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::{prelude::*, InputManagerBundle};

use self::{
    hover_effect::*,
    input::*,
    mesh::*,
    ui::{InspectorSelection, UiState},
};
use crate::line_material::LineMaterial;

pub mod hover_effect;
pub mod input;
pub mod mesh;
pub mod scene;
pub mod ui;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(DefaultInspectorConfigPlugin)
            .add_plugin(ui::EditorUiPlugin)
            .add_plugin(scene::ScenePlugin)
            .init_resource::<ui::UiTreeState>()
            .init_resource::<ui::UiState>()
            .init_resource::<CursorHoveringEntity>()
            .register_type::<CursorHoveringEntity>()
            .register_asset_reflect::<LineMaterial>()
            .add_startup_system(spawn_window)
            .add_startup_system(setup_effect)
            .add_system(camera_follow_focus)
            .add_system(move_camera_focus)
            .add_system(grab_cursor_on_move)
            .add_system(click_to_select)
            .add_system(set_hover_effect.after(click_to_select))
            .add_system(update_hover_entity)
            .add_system(spawn_point)
            .add_system(spawn_line)
            .add_system(update_lines)
            .add_event::<Solidify>()
            .add_event::<DeleteConnectedLines>()
            .add_system(solidify)
            .add_system(delete_connected_lines);
    }
}

#[derive(Component)]
pub struct EditorWindow;

#[derive(Component)]
pub struct EditorCamera;

#[derive(Component)]
pub struct EditorCameraFocus {
    distance: f32,
}

#[derive(Resource, Default, Reflect, FromReflect)]
pub struct CursorHoveringEntity {
    pub entity: Option<Entity>,
}

pub fn spawn_window(mut commands: Commands) {
    let editor_window = commands
        .spawn((
            Window {
                present_mode: PresentMode::AutoNoVsync,
                mode: WindowMode::Windowed,
                resolution: WindowResolution::new(1920., 1080.),
                position: WindowPosition::Centered(MonitorSelection::Index(0)),
                title: "Emitter Editor".to_owned(),
                transparent: false,
                ..default()
            },
            EditorWindow,
        ))
        .id();

    commands.spawn((
        Name::new("Editor Camera"),
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                target: RenderTarget::Window(WindowRef::Entity(editor_window)),
                ..default()
            },
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::rgba(0., 0., 0., 0.)),
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_xyz(0., 1., 75.).looking_at(Vec3::ZERO, Vec3::Z),
            ..default()
        },
        BloomSettings::default(),
        EditorCamera,
    ));

    commands.spawn((
        Name::new("Editor Camera Focus"),
        // Transform::from_xyz(0., 0., 0.).with_rotation(Quat::from_rotation_z(0.)),
        Transform::default(),
        EditorCameraFocus { distance: 75. },
        InputManagerBundle::<EditorAction> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(MouseButton::Left, EditorAction::Select)
                .insert(MouseButton::Right, EditorAction::Rotate)
                .insert(SingleAxis::mouse_wheel_y(), EditorAction::Zoom)
                .insert(
                    VirtualDPad {
                        up: KeyCode::W.into(),
                        down: KeyCode::S.into(),
                        left: KeyCode::A.into(),
                        right: KeyCode::D.into(),
                    },
                    EditorAction::Move,
                )
                .insert(KeyCode::R, EditorAction::SpawnLine)
                .insert(KeyCode::T, EditorAction::SpawnHandle)
                .build(),
        },
    ));
}

fn camera_follow_focus(
    focus: Query<(&Transform, &EditorCameraFocus), Without<EditorCamera>>,
    mut camera: Query<&mut Transform, With<EditorCamera>>,
) {
    let (focus_transform, focus) = focus.single();

    for mut camera_transform in camera.iter_mut() {
        *camera_transform =
            focus_transform.mul_transform(Transform::from_xyz(0., 0., focus.distance));
    }
}

fn move_camera_focus(
    mut query: Query<(&mut Transform, &ActionState<EditorAction>, &mut EditorCameraFocus)>,
    mut mouse_events: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    for (mut transform, action_state, mut focus) in query.iter_mut() {
        let up = transform.up();
        let right = transform.right();

        if action_state.pressed(EditorAction::Move) {
            let axis_pair = action_state.clamped_axis_pair(EditorAction::Move).unwrap();
            let vec = axis_pair.xy();
            transform.translation += right * vec.x * time.delta_seconds() * 20.;

            let up = Vec3::new(up.x, up.y, 0.).normalize();
            transform.translation += up * vec.y * time.delta_seconds() * 20.;
        }

        if action_state.pressed(EditorAction::Rotate) {
            for event in mouse_events.iter() {
                let delta = event.delta;
                let vec = Vec2::new(delta.x, delta.y);

                transform.rotate(Quat::from_rotation_z(vec.x * time.delta_seconds() * -0.5));
                transform.rotate(Quat::from_axis_angle(right, vec.y * time.delta_seconds() * -0.5));
            }
        }

        if action_state.just_pressed(EditorAction::Zoom) {
            let axis = action_state.value(EditorAction::Zoom);
            focus.distance -= axis * time.delta_seconds() * 2000.;

            if focus.distance < 1. {
                focus.distance = 1.;
            }
        }
    }
}

fn grab_cursor_on_move(
    query: Query<&ActionState<EditorAction>>,
    mut window: Query<&mut Window, With<EditorWindow>>,
    camera: Query<&mut Camera, With<EditorCamera>>,
) {
    let mut window = window.single_mut();
    let action_state = query.single();

    if action_state.just_pressed(EditorAction::Rotate) {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    } else if action_state.just_released(EditorAction::Rotate) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;

        let window_height = window.physical_height() as f64;
        window.set_physical_cursor_position(
            camera
                .single()
                .viewport
                .as_ref()
                .map(|v| v.physical_position + v.physical_size / 2)
                .map(|v| DVec2::new(v.x as f64, v.y as f64 * -1. + window_height)),
        );
    }
}

// fn toggle_debug_render(
//     mut debug_render: ResMut<DebugRenderContext>,
//     input: Query<&ActionState<EditorAction>>,
//     ui_state: Res<UiState>,
// ) {
//     if !ui_state.hovering_camera {
//         return;
//     }

//     debug_render.enabled = input.single().pressed(EditorAction::Select);
// }

fn update_hover_entity(
    window: Query<&Window, With<EditorWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    context: Res<RapierContext>,
    mut hovering_entity: ResMut<CursorHoveringEntity>,
    mut ui_state: ResMut<UiState>,
) {
    let Ok(window) = window.get_single() else { return };
    let (camera, camera_transform) = camera.single();

    let Some(cursor_position) = window.cursor_position() else {
        hovering_entity.entity = None;
        ui_state.hovering_camera = false;
        return;
    };

    let viewport = camera.viewport.as_ref().unwrap();

    let cursor_position = cursor_position
        - Vec2::new(
            viewport.physical_position.x as f32,
            (window.physical_height() - (viewport.physical_position.y + viewport.physical_size.y))
                as f32,
        );

    if cursor_position.x < 0.
        || cursor_position.x > viewport.physical_size.x as f32
        || cursor_position.y < 0.
        || cursor_position.y > viewport.physical_size.y as f32
    {
        hovering_entity.entity = None;
        ui_state.hovering_camera = false;
        return;
    }

    ui_state.hovering_camera = true;

    let Some(ray) = camera.viewport_to_world(
        camera_transform,
        cursor_position,
    ) else {
        hovering_entity.entity = None;
        return;
    };

    hovering_entity.entity = context
        .cast_ray(ray.origin, ray.direction, 1000., true, QueryFilter::new())
        .map(|(entity, _toi)| entity);
}

fn click_to_select(
    hovering_entity: Res<CursorHoveringEntity>,
    input: Query<&ActionState<EditorAction>>,
    mut ui_state: ResMut<UiState>,
) {
    if !ui_state.hovering_camera {
        return;
    }

    if input.single().just_released(EditorAction::Select) {
        if let Some(entity) = hovering_entity.entity {
            ui_state.selected_entities.select_replace(entity);
            ui_state.selection = InspectorSelection::Entities;
        } else {
            ui_state.selected_entities.clear();
            ui_state.selection = InspectorSelection::None;
        };
    }
}
