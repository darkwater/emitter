use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{collision_groups, line_material::LineStrip, LineMaterial};

pub fn spawn_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    // triangle
    let triangle = meshes.add(Mesh::from(LineStrip {
        points: vec![
            Vec3::ZERO,
            Vec3::X * 10.,
            Vec3::Z * 5.,
            Vec3::Y * 10.,
            Vec3::ZERO,
            Vec3::Z * 5.,
        ],
    }));

    let collider = Collider::trimesh(
        vec![
            Vec3::Z,
            -Vec3::Z,
            Vec3::X * 10. + Vec3::Z,
            Vec3::X * 10. + -Vec3::Z,
            Vec3::Y * 10. + Vec3::Z,
            Vec3::Y * 10. + -Vec3::Z,
        ],
        vec![[0, 2, 1], [1, 2, 3], [4, 0, 5], [5, 0, 1]],
    );

    commands.spawn((
        MaterialMeshBundle {
            mesh: triangle.clone(),
            transform: Transform::from_rotation(Quat::from_rotation_z(PI * 1.))
                .with_translation(Vec3::X * 18. + Vec3::Y * 18.),
            material: materials.add(LineMaterial { color: Color::CYAN }),
            ..default()
        },
        RigidBody::Fixed,
        CollisionGroups::new(collision_groups::WALL, collision_groups::ALL),
        collider.clone(),
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: triangle.clone(),
            transform: Transform::from_rotation(Quat::from_rotation_z(PI * 1.5))
                .with_translation(Vec3::X * -18. + Vec3::Y * 18.),
            material: materials.add(LineMaterial { color: Color::CYAN }),
            ..default()
        },
        RigidBody::Fixed,
        CollisionGroups::new(collision_groups::WALL, collision_groups::ALL),
        collider.clone(),
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: triangle.clone(),
            transform: Transform::from_rotation(Quat::from_rotation_z(PI * 0.))
                .with_translation(Vec3::X * -18. + Vec3::Y * -18.),
            material: materials.add(LineMaterial { color: Color::CYAN }),
            ..default()
        },
        RigidBody::Fixed,
        collider.clone(),
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: triangle.clone(),
            transform: Transform::from_rotation(Quat::from_rotation_z(PI * 0.5))
                .with_translation(Vec3::X * 18. + Vec3::Y * -18.),
            material: materials.add(LineMaterial { color: Color::CYAN }),
            ..default()
        },
        RigidBody::Fixed,
        collider.clone(),
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: triangle.clone(),
            transform: Transform::from_rotation(Quat::from_rotation_z(PI * 1.25))
                .with_translation(Vec3::Y * 40.)
                .with_scale(Vec3::new(3., 3., 1.5)),
            material: materials.add(LineMaterial { color: Color::BLUE }),
            ..default()
        },
        RigidBody::Fixed,
        collider.clone(),
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: triangle.clone(),
            transform: Transform::from_rotation(Quat::from_rotation_z(PI * 1.75))
                .with_translation(Vec3::X * -40.)
                .with_scale(Vec3::new(3., 3., 1.5)),
            material: materials.add(LineMaterial { color: Color::BLUE }),
            ..default()
        },
        RigidBody::Fixed,
        collider.clone(),
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: triangle.clone(),
            transform: Transform::from_rotation(Quat::from_rotation_z(PI * 0.25))
                .with_translation(Vec3::Y * -40.)
                .with_scale(Vec3::new(3., 3., 1.5)),
            material: materials.add(LineMaterial { color: Color::BLUE }),
            ..default()
        },
        RigidBody::Fixed,
        collider.clone(),
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: triangle,
            transform: Transform::from_rotation(Quat::from_rotation_z(PI * 0.75))
                .with_translation(Vec3::X * 40.)
                .with_scale(Vec3::new(3., 3., 1.5)),
            material: materials.add(LineMaterial { color: Color::BLUE }),
            ..default()
        },
        RigidBody::Fixed,
        collider,
    ));
}
