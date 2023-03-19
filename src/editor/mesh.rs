use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
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

pub fn spawn_point(
    input: Query<&ActionState<EditorAction>>,
    window: Query<&Window, With<EditorWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    mut ui_state: ResMut<UiState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    if !input.single().just_pressed(EditorAction::SpawnHandle) {
        return;
    }

    let Ok(window) = window.get_single() else { return };
    let (camera, camera_transform) = camera.single();

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let viewport = camera.viewport.as_ref().unwrap();

    let Some(ray) = camera.viewport_to_world(
        camera_transform,
        cursor_position - Vec2::new(
            viewport.physical_position.x as f32,
            (
                window.physical_height() -
                (viewport.physical_position.y + viewport.physical_size.y)
            ) as f32,
        ),
    ) else {
        return;
    };

    let Some(distance) = ray.intersect_plane(Vec3::ZERO, Vec3::Z) else {
        return;
    };

    let position = ray.get_point(distance).round();

    let entity = commands
        .spawn((
            MeshPoint,
            // Transform::from_translation(position),
            // GlobalTransform::default(),
            Collider::ball(1.),
            CollisionGroups::new(collision_groups::EDITOR_HANDLE, collision_groups::NONE),
            HoverEffect,
            MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(LineList {
                    lines: vec![
                        (Vec3::NEG_X * 0.3, Vec3::X * 0.3),
                        (Vec3::NEG_Y * 0.3, Vec3::Y * 0.3),
                        (Vec3::NEG_Z * 0.3, Vec3::Z * 0.3),
                    ],
                })),
                transform: Transform::from_translation(position),
                material: materials.add(LineMaterial { color: Color::PURPLE }),
                ..default()
            },
        ))
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
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    if !input.single().just_pressed(EditorAction::SpawnLine) {
        return;
    }

    let UiState { ref mut selected_entities, .. } = *ui_state;
    let Some(end) = hover_target.entity else { return };
    let Some(start) = selected_entities.iter().next() else { return };

    if start == end {
        return;
    }

    if mesh_points.get(start).is_err() || mesh_points.get(end).is_err() {
        return;
    }

    commands.spawn((MeshLine { start, end }, MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(LineList { lines: vec![(Vec3::ZERO, Vec3::ONE)] })),
        transform: Transform::from_xyz(0., 0., 0.),
        material: materials.add(LineMaterial { color: Color::PURPLE }),
        ..default()
    }));

    selected_entities.select_replace(end);
}

pub fn update_lines(
    mut lines: Query<(&mut Transform, &Handle<LineMaterial>, &MeshLine)>,
    entities: Query<&Transform, Without<MeshLine>>,
    ui_state: Res<UiState>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    for (mut transform, material, line) in lines.iter_mut() {
        let start = entities.get(line.start).unwrap();
        let end = entities.get(line.end).unwrap();

        transform.translation = start.translation;
        transform.scale = end.translation - start.translation;

        materials.get_mut(material).unwrap().color = ui_state.new_mesh_props.color();
    }
}

pub struct Solidify;

pub fn solidify(
    mut event_reader: EventReader<Solidify>,
    point_query: Query<(Entity, &Transform, &MeshPoint)>,
    line_query: Query<(Entity, &MeshLine)>,
    ui_state: Res<UiState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
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
        MaterialMeshBundle {
            mesh: meshes.add(LineList { lines }.into()),
            material: materials.add(LineMaterial {
                color: ui_state.new_mesh_props.color(),
            }),
            ..default()
        },
        RigidBody::Fixed,
        CollisionGroups::new(collision_groups::WALL, collision_groups::ALL),
        Collider::trimesh(collider_vertices, collider_indices),
    ));
}

pub struct DeleteConnectedLines;

pub fn delete_connected_lines(
    mut event_reader: EventReader<DeleteConnectedLines>,
    mut lines: Query<(Entity, &MeshLine)>,
    mut ui_state: ResMut<UiState>,
    mut commands: Commands,
) {
    // let mut query = line_query
    //     .iter(self.world)
    //     .filter(|(_entity, line)| {
    //         line.start == self.selected_entities.as_slice()[0]
    //             || line.end == self.selected_entities.as_slice()[0]
    //     })
    //     .peekable();

    // if query.peek().is_some() {
    //     let button = ui.button("Delete lines");
    //     if button.clicked() {
    //         for (entity, _line) in query {
    //             self.world.despawn(entity);
    //         }
    //     }
    // }

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
