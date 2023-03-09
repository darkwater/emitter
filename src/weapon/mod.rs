use bevy::prelude::*;
use bevy_rapier3d::prelude::{RigidBody, Velocity};

use crate::{bullet::Bullet, utils::zlock::ZLocked, LineList, LineMaterial};

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(shoot);
    }
}

#[derive(Component, Default)]
pub struct WeaponTrigger(pub bool);

#[derive(Component)]
pub struct Weapon {
    pub cooldown: f64,
    pub next_shot: f64,
    pub damage: f32,
    pub velocity: f32,
    pub spread: f32,
    pub range: f32,
    pub color: Color,
}

pub fn shoot(
    mut query: Query<(&Transform, &mut Weapon, &mut WeaponTrigger)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
    time: Res<Time>,
) {
    for (transform, mut weapon, mut trigger) in query.iter_mut() {
        if !trigger.0 {
            continue;
        }

        trigger.0 = false;

        let Weapon {
            cooldown,
            ref mut next_shot,
            damage,
            velocity,
            spread,
            range,
            color,
        } = *weapon;

        if time.elapsed_seconds_f64() < *next_shot {
            continue;
        }

        *next_shot = time.elapsed_seconds_f64() + cooldown;

        let angle = transform.rotation * Quat::from_rotation_z(spread * rand::random::<f32>());
        let direction = angle * Vec3::X;

        commands.spawn((
            Name::new("Bullet"),
            MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(LineList {
                    lines: vec![(Vec3::X * -0.5, Vec3::X * 0.5)],
                })),
                transform: *transform,
                // .with_translation(transform.translation + direction),
                material: materials.add(LineMaterial { color }),
                ..default()
            },
            RigidBody::Dynamic,
            Velocity {
                linvel: direction * velocity,
                angvel: Vec3::ZERO,
            },
            ZLocked { angular: true },
            Bullet { damage },
        ));
    }
}
