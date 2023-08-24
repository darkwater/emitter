use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use itertools::Itertools;
use leafwing_input_manager::prelude::*;

use super::{
    hover_effect::HoverEffect, input::EditorAction, ui::UiState, CursorHoveringEntity,
    EditorCamera, EditorWindow,
};
use crate::{
    collision_groups,
    line_material::{LineList, LineMaterial},
};

#[derive(Component)]
pub struct MeshPoint;

#[derive(Component)]
pub struct MeshLine {
    pub start: Entity,
    pub end: Entity,
}

#[derive(Bundle)]
pub struct MeshPointBundle {
    pub mesh_point: MeshPoint,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub hover_effect: HoverEffect,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

impl MeshPointBundle {
    fn new(asset_server: &AssetServer) -> Self {
        Self {
            mesh_point: MeshPoint,
            collider: Collider::ball(1.),
            collision_groups: CollisionGroups::new(
                collision_groups::EDITOR_HANDLE,
                collision_groups::NONE,
            ),
            hover_effect: HoverEffect,
            // material_mesh_bundle: MaterialMeshBundle {
            //     mesh: meshes.add(Mesh::from(LineList {
            //         lines: vec![
            //             (Vec3::NEG_X * 0.3, Vec3::X * 0.3),
            //             (Vec3::NEG_Y * 0.3, Vec3::Y * 0.3),
            //             (Vec3::NEG_Z * 0.3, Vec3::Z * 0.3),
            //         ],
            //     })),
            //     transform: Transform::from_translation(position),
            //     material: materials.add(LineMaterial { color: Color::PURPLE }),
            //     ..default()
            // },
            mesh: asset_server.load("models/editor-handle.mdl.ron"),
            material: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            computed_visibility: Default::default(),
        }
    }
}

pub fn spawn_point(
    input: Query<&ActionState<EditorAction>>,
    window: Query<&Window, With<EditorWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    mut ui_state: ResMut<UiState>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<LineMaterial>>,
) {
    if !input.single().just_pressed(EditorAction::SpawnHandle) {
        return;
    }

    let Ok(window) = window.get_single() else {
        return;
    };
    let (camera, camera_transform) = camera.single();

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let viewport = camera.viewport.as_ref().unwrap();

    let Some(ray) = camera.viewport_to_world(
        camera_transform,
        cursor_position
            - Vec2::new(
                viewport.physical_position.x as f32,
                (window.physical_height()
                    - (viewport.physical_position.y + viewport.physical_size.y))
                    as f32,
            ),
    ) else {
        return;
    };

    let Some(distance) = ray.intersect_plane(Vec3::ZERO, Vec3::Z) else {
        return;
    };

    let position = ray.get_point(distance).round();

    let entity = commands
        // .spawn((
        //     MeshPoint,
        //     // Transform::from_translation(position),
        //     // GlobalTransform::default(),
        //     Collider::ball(1.),
        //     CollisionGroups::new(collision_groups::EDITOR_HANDLE, collision_groups::NONE),
        //     HoverEffect,
        //     MaterialMeshBundle {
        //         mesh: meshes.add(Mesh::from(LineList {
        //             lines: vec![
        //                 (Vec3::NEG_X * 0.3, Vec3::X * 0.3),
        //                 (Vec3::NEG_Y * 0.3, Vec3::Y * 0.3),
        //                 (Vec3::NEG_Z * 0.3, Vec3::Z * 0.3),
        //             ],
        //         })),
        //         transform: Transform::from_translation(position),
        //         material: materials.add(LineMaterial { color: Color::PURPLE }),
        //         ..default()
        //     },
        // ))
        .spawn(MeshPointBundle {
            transform: Transform::from_translation(position),
            ..MeshPointBundle::new(&asset_server)
        })
        .id();

    ui_state.selected_entities.select_replace(entity);
}

pub fn spawn_line(
    input: Query<&ActionState<EditorAction>>,
    hover_target: Res<CursorHoveringEntity>,
    mesh_points: Query<&MeshPoint>,
    mut ui_state: ResMut<UiState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if !input.single().just_pressed(EditorAction::SpawnLine) {
        return;
    }

    let UiState { ref mut selected_entities, .. } = *ui_state;
    let Some(end) = hover_target.entity else {
        return;
    };
    let Some(start) = selected_entities.iter().next() else {
        return;
    };

    if start == end {
        return;
    }

    if mesh_points.get(start).is_err() || mesh_points.get(end).is_err() {
        return;
    }

    commands.spawn((MeshLine { start, end }, MaterialMeshBundle::<StandardMaterial> {
        mesh: meshes.add(Mesh::from(LineList {
            lines: vec![(Vec3::ZERO, Vec3::ONE)],
            color: Color::WHITE,
        })),
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    }));

    selected_entities.select_replace(end);
}

pub fn update_lines(
    mut lines: Query<(Entity, &mut Transform, &Handle<LineMaterial>, &MeshLine)>,
    entities: Query<&Transform, Without<MeshLine>>,
    ui_state: Res<UiState>,
    mut materials: ResMut<Assets<LineMaterial>>,
    mut commands: Commands,
) {
    for (entity, mut transform, material, line) in lines.iter_mut() {
        let Ok((start, end)) = entities
            .get(line.start)
            .and_then(|start| entities.get(line.end).map(|end| (start, end)))
        else {
            commands.entity(entity).despawn();
            continue;
        };

        transform.translation = start.translation;
        transform.scale = end.translation - start.translation;

        if let Some(material) = materials.get_mut(material) {
            material.color = ui_state.new_mesh_props.color();
        }
    }
}

#[derive(Event)]
pub struct ExplodeMesh;

pub fn explode_mesh(
    mut event_reader: EventReader<ExplodeMesh>,
    walls: Query<(&Transform, &Handle<LineMaterial>, &Handle<Mesh>), With<WallMesh>>,
    ui_state: ResMut<UiState>,
    asset_server: Res<AssetServer>,
    materials: Res<Assets<LineMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    if event_reader.iter().next().is_none() {
        return;
    }

    for entity in ui_state.selected_entities.iter() {
        let (transform, line_material, mesh) = walls.get(entity).unwrap();

        let mesh = meshes.get(mesh).unwrap();
        let line_material = materials.get(line_material).unwrap();

        let Some(attr) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
            continue;
        };

        let points = attr
            .as_float3()
            .unwrap()
            .iter()
            .map(|p| Vec3::from(*p))
            .collect::<Vec<_>>();

        let mut point_entities = vec![];

        for point in &points {
            let position = Transform::from_translation(transform.transform_point(*point));

            if point_entities.iter().any(|&(pos, _ent)| pos == position) {
                continue;
            }

            point_entities.push((
                position,
                commands
                    .spawn(MeshPointBundle {
                        transform: position,
                        ..MeshPointBundle::new(&asset_server)
                    })
                    .id(),
            ));
        }

        let lines = points.iter().tuple_windows();

        for (from, to) in lines {
            let (_, from_ent) = point_entities
                .iter()
                .find(|(pos, _ent)| pos.translation == *from)
                .unwrap();

            let (_, to_ent) = point_entities
                .iter()
                .find(|(pos, _ent)| pos.translation == *to)
                .unwrap();

            commands.spawn((MeshLine { start: *from_ent, end: *to_ent }, MaterialMeshBundle::<
                StandardMaterial,
            > {
                mesh: meshes.add(Mesh::from(LineList {
                    lines: vec![(Vec3::ZERO, Vec3::ONE)],
                    color: line_material.color,
                })),
                transform: Transform::from_xyz(0., 0., 0.),
                ..default()
            }));
        }

        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Event)]
pub struct Solidify;

#[derive(Component)]
pub struct WallMesh;

pub fn solidify(
    mut event_reader: EventReader<Solidify>,
    point_query: Query<(Entity, &Transform, &MeshPoint)>,
    line_query: Query<(Entity, &MeshLine)>,
    ui_state: Res<UiState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    if event_reader.iter().next().is_none() {
        return;
    }

    let mut lines = vec![];
    let mut collider_vertices = vec![];
    let mut collider_indices = vec![];

    for (entity, mesh_line) in line_query.iter() {
        let from = point_query.get(mesh_line.start).unwrap();
        let to = point_query.get(mesh_line.end).unwrap();

        lines.push((from.1.translation, to.1.translation));

        if from.1.translation.z == 0.0 && to.1.translation.z == 0.0 {
            let idx = collider_vertices.len() as u32;

            collider_vertices.extend_from_slice(&[
                from.1.translation - Vec3::Z,
                from.1.translation + Vec3::Z,
                to.1.translation - Vec3::Z,
                to.1.translation + Vec3::Z,
            ]);

            collider_indices.push([idx, idx + 2, idx + 1]);
            collider_indices.push([idx + 1, idx + 2, idx + 3]);
        }

        commands.entity(entity).despawn_recursive();
    }

    for (entity, _, _) in point_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    commands.spawn((
        MaterialMeshBundle::<StandardMaterial> {
            mesh: meshes.add(
                LineList {
                    lines,
                    color: ui_state.new_mesh_props.color(),
                }
                .into(),
            ),
            ..default()
        },
        RigidBody::Fixed,
        CollisionGroups::new(collision_groups::WALL, collision_groups::ALL),
        Collider::trimesh(collider_vertices, collider_indices),
        WallMesh,
    ));
}

#[derive(Event)]
pub struct DeleteConnectedLines;

pub fn delete_connected_lines(
    mut event_reader: EventReader<DeleteConnectedLines>,
    mut lines: Query<(Entity, &MeshLine)>,
    mut ui_state: ResMut<UiState>,
    mut commands: Commands,
) {
    if event_reader.iter().next().is_none() {
        return;
    }

    let UiState { ref mut selected_entities, .. } = *ui_state;

    for entity in selected_entities.iter() {
        for (line_entity, line) in lines.iter_mut() {
            if line.start == entity || line.end == entity {
                commands.entity(line_entity).despawn_recursive();
            }
        }
    }
}
