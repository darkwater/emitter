use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

use super::{input::PlayerAction, PlayerAimTarget, PlayerShip, ShipEngine};
use crate::{utils::look_at_2d::LookAt2d, weapon::WeaponTrigger, PlayerWindow, CAMERA_OFFSET};

pub fn move_player_ship(
    mut query: Query<(&mut ShipEngine, &ActionState<PlayerAction>), With<PlayerShip>>,
) {
    for (mut engine, action_state) in query.iter_mut() {
        if action_state.pressed(PlayerAction::Move) {
            let axis_pair = action_state.clamped_axis_pair(PlayerAction::Move).unwrap();
            let vec = axis_pair.xy().clamp_length_max(1.);
            engine.target_velocity = (vec * 25.).extend(0.);
        } else {
            engine.target_velocity = Vec3::splat(0.);
        }
    }
}

#[derive(Component)]
pub struct PlayerFollower;

pub fn follow_player_ship(
    mut query: Query<(&Transform, &PlayerShip), Without<Camera3d>>,
    mut camera: Query<(&mut Transform, &Camera3d), With<PlayerFollower>>,
) {
    for (transform, _) in query.iter_mut() {
        for (mut camera_transform, _) in camera.iter_mut() {
            camera_transform.translation = camera_transform
                .translation
                .lerp(transform.translation + CAMERA_OFFSET, 0.1);
        }
    }
}

pub fn apply_ship_engine(mut query: Query<(&mut Velocity, &ShipEngine)>, time: Res<Time>) {
    for (mut velocity, engine) in query.iter_mut() {
        velocity.linvel = velocity
            .linvel
            .lerp(engine.target_velocity, engine.power * time.delta_seconds());

        velocity.angvel = Vec3::ZERO;
    }
}

pub fn shoot(mut query: Query<(&PlayerShip, &ActionState<PlayerAction>, &mut WeaponTrigger)>) {
    for (_, action_state, mut trigger) in query.iter_mut() {
        if action_state.pressed(PlayerAction::Shoot) {
            trigger.0 = true;
        }
    }
}

pub fn move_aim_target(
    window: Query<&Window, With<PlayerWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<PlayerFollower>>,
    mut player_aim_target: Query<&mut Transform, With<PlayerAimTarget>>,
) {
    let Ok(window) = window.get_single() else { return };
    let (camera, camera_transform) = camera.single();
    let mut player_aim_target = player_aim_target.single_mut();

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Some(ray) = camera
        .viewport_to_world(camera_transform, cursor_position) else {
            return;
        };

    let Some(distance) = ray.intersect_plane(Vec3::ZERO, Vec3::Z) else {
        return;
    };

    let position = ray.get_point(distance);

    player_aim_target.translation = position;
}

pub fn aim_player_ship(
    mut player_ship: Query<&mut Transform, (With<PlayerShip>, Without<PlayerAimTarget>)>,
    player_aim_target: Query<&Transform, (With<PlayerAimTarget>, Without<PlayerShip>)>,
) {
    let mut player_ship = player_ship.single_mut();
    let player_aim_target = player_aim_target.single();

    player_ship.look_at_2d(player_aim_target.translation);
}
