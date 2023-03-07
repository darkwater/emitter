use std::f32::consts::PI;

use bevy::prelude::*;

use super::{PlayerShip, ShipEngine};
use crate::{utils::drawing::arc, Inertia, LineList, LineMaterial};

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    // Spawn a list of lines with start and end points for each lines
    let mut lines = vec![
        (Vec3::new(1., 0., 0.), Vec3::new(-0.8, 0.5, 0.)),
        (Vec3::new(-0.8, 0.5, 0.), Vec3::new(-0.5, 0., 0.)),
        (Vec3::new(-0.5, 0., 0.), Vec3::new(-0.8, -0.5, 0.)),
        (Vec3::new(-0.8, -0.5, 0.), Vec3::new(1., 0., 0.)),
    ];

    lines.append(&mut arc(1., PI * 0.15, PI * 0.65, 8, true));
    lines.append(&mut arc(1., PI * -0.15, PI * -0.65, 8, true));

    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(LineList { lines })),
            transform: Transform::from_xyz(0., 0., 0.),
            material: materials.add(LineMaterial { color: Color::ORANGE * 5. }),
            ..default()
        },
        PlayerShip,
        Inertia::default(),
        ShipEngine { power: 25., ..Default::default() },
    ));
}
