use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

use super::{input::PlayerAction, PlayerAimTarget, PlayerShip, ShipEngine};
use crate::{
    collision_groups,
    line_material::LineList,
    team::Team,
    utils::{drawing::arc, zlock::ZLocked},
    weapon::{Weapon, WeaponTrigger},
    LineMaterial,
};

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    let mut lines = vec![
        (Vec3::new(1., 0., 0.), Vec3::new(-0.8, 0.5, 0.)),
        (Vec3::new(-0.8, 0.5, 0.), Vec3::new(-0.5, 0., 0.)),
        (Vec3::new(-0.5, 0., 0.), Vec3::new(-0.8, -0.5, 0.)),
        (Vec3::new(-0.8, -0.5, 0.), Vec3::new(1., 0., 0.)),
    ];

    lines.append(&mut arc(1., PI * 0.15, PI * 0.65, 8, true));
    lines.append(&mut arc(1., PI * -0.15, PI * -0.65, 8, true));

    commands.spawn((
        Name::new("Player Ship"),
        PlayerShip,
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(LineList { lines })),
            transform: Transform::from_xyz(0., 0., 0.),
            material: materials.add(LineMaterial { color: Color::ORANGE * 5. }),
            ..default()
        },
        ShipEngine { power: 40., ..Default::default() },
        RigidBody::Dynamic,
        Velocity::default(),
        Collider::ball(1.),
        CollisionGroups::new(collision_groups::PLAYER, collision_groups::ALL),
        ZLocked { angular: true },
        Weapon {
            cooldown: 0.2,
            next_shot: 0.,
            damage: 1.,
            velocity: 50.,
            spread: 0.,
            color: Color::RED * 5.,
        },
        WeaponTrigger::default(),
        Team::Player,
        InputManagerBundle::<PlayerAction> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(DualAxis::left_stick(), PlayerAction::Move)
                .insert(DualAxis::right_stick(), PlayerAction::Aim)
                .insert(DualAxis::right_stick(), PlayerAction::Shoot)
                .insert(MouseButton::Left, PlayerAction::Shoot)
                .insert(
                    VirtualDPad {
                        up: KeyCode::W.into(),
                        down: KeyCode::S.into(),
                        left: KeyCode::A.into(),
                        right: KeyCode::D.into(),
                    },
                    PlayerAction::Move,
                )
                .build(),
        },
    ));

    commands.spawn((
        Name::new("Player Aim Target"),
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(LineList {
                lines: vec![
                    (Vec3::new(-0.2, -0.2, 0.), Vec3::new(0.2, 0.2, 0.)),
                    (Vec3::new(0.2, -0.2, 0.), Vec3::new(-0.2, 0.2, 0.)),
                ],
            })),
            transform: Transform::from_translation(Vec3::X * 9999.),
            material: materials.add(LineMaterial { color: Color::ORANGE * 5. }),
            ..default()
        },
        PlayerAimTarget,
    ));
}
