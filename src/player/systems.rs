use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::{PlayerAimTarget, PlayerShip, ShipEngine};
use crate::{
    bullet::Bullet,
    utils::{look_at_2d::LookAt2d, zlock::ZLocked},
    LineList, LineMaterial, CAMERA_OFFSET,
};

pub fn move_player_ship(
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut ShipEngine, With<PlayerShip>>,
) {
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

    let mut vec = Vec3::new(x_dir, y_dir, 0.);

    if vec.length() > 0. {
        vec = vec.normalize();
    }

    for mut engine in query.iter_mut() {
        engine.target_velocity = vec * 25.;
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

        velocity.angvel = Vec3::ZERO;
    }
}

pub fn shoot(
    input: ResMut<Input<MouseButton>>,
    mut query: Query<(&Transform, &PlayerShip)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    if input.just_pressed(MouseButton::Left) {
        for (transform, _) in query.iter_mut() {
            let direction = transform.right();

            commands.spawn((
                MaterialMeshBundle {
                    mesh: meshes.add(Mesh::from(LineList {
                        lines: vec![(Vec3::X * -0.5, Vec3::X * 0.5)],
                    })),
                    transform: *transform,
                    // .with_translation(transform.translation + direction),
                    material: materials.add(LineMaterial { color: Color::RED * 5. }),
                    ..default()
                },
                RigidBody::Dynamic,
                Velocity {
                    linvel: direction * 50.,
                    angvel: Vec3::ZERO,
                },
                ZLocked { angular: true },
                Bullet { damage: 1. },
            ));
        }
    }
}

pub fn move_aim_target(
    window: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut player_aim_target: Query<&mut Transform, With<PlayerAimTarget>>,
) {
    let window = window.single();
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
