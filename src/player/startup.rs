use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

use super::{input::PlayerAction, PlayerAimTarget, PlayerShip, ShipEngine};
use crate::{
    collision_groups,
    team::Team,
    utils::zlock::ZLocked,
    weapon::{Weapon, WeaponTrigger},
};

pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animations: ResMut<Assets<AnimationClip>>,
) {
    let name = Name::new("Player ship");

    let mesh_name = Name::new("Player ship model");
    let mut animation = AnimationClip::default();
    animation.add_curve_to_path(EntityPath { parts: vec![mesh_name.clone()] }, VariableCurve {
        keyframe_timestamps: vec![0., 2., 4.],
        keyframes: Keyframes::Rotation(vec![
            Quat::from_rotation_x(0.),
            Quat::from_rotation_x(PI * 1.),
            Quat::from_rotation_x(PI * 2.),
        ]),
    });

    let mut animation_player = AnimationPlayer::default();
    animation_player.play(animations.add(animation)).repeat();

    let mesh = commands
        .spawn((
            mesh_name,
            MaterialMeshBundle::<StandardMaterial> {
                mesh: asset_server.load("models/ship.mdl.ron"),
                ..default()
            },
            animation_player,
        ))
        .id();

    commands
        .spawn((
            name,
            PlayerShip,
            TransformBundle { ..default() },
            VisibilityBundle { ..default() },
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
        ))
        .add_child(mesh);

    commands.spawn((
        Name::new("Player Aim Target"),
        MaterialMeshBundle::<StandardMaterial> {
            mesh: asset_server.load("models/ship-target.mdl.ron"),
            transform: Transform::from_translation(Vec3::X * 9999.),
            ..default()
        },
        PlayerAimTarget,
    ));
}
