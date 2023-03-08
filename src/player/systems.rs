use bevy::{prelude::*, utils::petgraph::matrix_graph::Zero};
use bevy_rapier2d::prelude::*;

use super::{PlayerShip, ShipEngine};
use crate::{Inertia, CAMERA_OFFSET};

pub fn move_player_ship(
    input: Res<Input<KeyCode>>,
    mut query: Query<(&mut ShipEngine, &mut Transform), With<PlayerShip>>,
    mut window_q: Query<&mut Window>,
    time: Res<Time>,
) {
    let window = window_q.single_mut();

    let mut x_dir = 0.;
    let mut y_dir = 0.;

    if input.pressed(KeyCode::W) {
        y_dir += 1.;
    }

    if input.pressed(KeyCode::S) {
        y_dir -= 1.;
    }

    if input.pressed(KeyCode::A) {
        x_dir -= 1.;
    }

    if input.pressed(KeyCode::D) {
        x_dir += 1.;
    }

    let mut vec = Vec2::new(x_dir, y_dir);

    if vec.length() > 0. {
        vec = vec.normalize();
    }

    for (mut engine, mut transform) in query.iter_mut() {
        engine.target_velocity = vec * 25.;

        if let Some(cursor_pos) = window.cursor_position() {
            let center = Vec2::new(window.width() / 2., window.height() / 2.);
            let target = Vec2::new(cursor_pos.x, cursor_pos.y) - center;

            let angle = Vec2::X.angle_between(target);

            if angle.is_normal() || angle.is_zero() {
                transform.rotation = Quat::from_rotation_z(angle);
            }
        }
    }
}

pub fn follow_player_ship(
    mut query: Query<(&Transform, &PlayerShip), Without<Camera3d>>,
    mut camera: Query<(&mut Transform, &Camera3d)>,
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

        velocity.angvel = 0.;
    }
}
