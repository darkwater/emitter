use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use super::{input::EditorAction, ui::UiState, CursorHoveringEntity};
use crate::line_material::{LineList, LineMaterial};

#[derive(Component)]
pub struct MeshPoint;

// #[derive(Component)]
// pub struct Mesh {
//     pub lines: Vec<(Entity, Entity)>,
// }

#[derive(Component)]
pub struct MeshLine {
    pub start: Entity,
    pub end: Entity,
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

    if mesh_points.get(start).is_err() || mesh_points.get(end).is_err() {
        return;
    }

    commands.spawn((MeshLine { start, end }, MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(LineList { lines: vec![(Vec3::ZERO, Vec3::ONE)] })),
        transform: Transform::from_xyz(0., 0., 0.),
        material: materials.add(LineMaterial { color: Color::PURPLE * 5. }),
        ..default()
    }));

    selected_entities.select_replace(end);
}

pub fn update_lines(
    mut lines: Query<(&mut Transform, &MeshLine)>,
    entities: Query<&Transform, Without<MeshLine>>,
) {
    for (mut transform, line) in lines.iter_mut() {
        let start = entities.get(line.start).unwrap();
        let end = entities.get(line.end).unwrap();

        transform.translation = start.translation;
        transform.scale = end.translation - start.translation;
    }
}
