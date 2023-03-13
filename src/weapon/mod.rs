use bevy::prelude::*;
use bevy_rapier3d::prelude::{RigidBody, Velocity};

use crate::{
    bullet::Bullet, line_material::LineList, team::Team, utils::zlock::ZLocked, LineMaterial,
};

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(shoot);
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct WeaponTrigger(pub bool);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Weapon {
    pub cooldown: f64,
    pub next_shot: f64,
    pub damage: f32,
    pub velocity: f32,
    pub spread: f32,
    pub color: Color,
}

#[derive(Bundle)]
pub struct BulletBundle {
    pub mesh: MaterialMeshBundle<LineMaterial>,
    pub body: RigidBody,
    pub velocity: Velocity,
    pub zlock: ZLocked,
    pub bullet: Bullet,
}

impl BulletBundle {
    pub fn new(
        transform: Transform,
        velocity: Velocity,
        damage: f32,
        color: Color,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<LineMaterial>>,
    ) -> Self {
        Self {
            mesh: MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(LineList {
                    lines: vec![(Vec3::X * -0.5, Vec3::X * 0.5)],
                })),
                transform,
                material: materials.add(LineMaterial { color }),
                ..default()
            },
            body: RigidBody::Dynamic,
            zlock: ZLocked { angular: true },
            bullet: Bullet { damage },
            velocity,
        }
    }
}

pub fn shoot(
    mut query: Query<(&Transform, &mut Weapon, &mut WeaponTrigger, Option<&Team>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
    time: Res<Time>,
) {
    for (transform, mut weapon, mut trigger, team) in query.iter_mut() {
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
            color,
        } = *weapon;

        if time.elapsed_seconds_f64() < *next_shot {
            continue;
        }

        *next_shot = time.elapsed_seconds_f64() + cooldown;

        let angle = transform.rotation * Quat::from_rotation_z(spread * rand::random::<f32>());
        let direction = angle * Vec3::X;

        // commands.spawn((
        //     Name::new("Bullet"),
        //     MaterialMeshBundle {
        //         mesh: meshes.add(Mesh::from(LineList {
        //             lines: vec![(Vec3::X * -0.5, Vec3::X * 0.5)],
        //         })),
        //         transform: *transform,
        //         // .with_translation(transform.translation + direction),
        //         material: materials.add(LineMaterial { color }),
        //         ..default()
        //     },
        //     RigidBody::Dynamic,
        //     Velocity {
        //         linvel: direction * velocity,
        //         angvel: Vec3::ZERO,
        //     },
        //     ZLocked { angular: true },
        //     Bullet { damage },
        // ));

        let bundle = BulletBundle::new(
            *transform,
            Velocity {
                linvel: direction * velocity,
                angvel: Vec3::ZERO,
            },
            damage,
            color,
            &mut meshes,
            &mut materials,
        );

        if let Some(team) = team {
            commands.spawn((bundle, *team));
        } else {
            commands.spawn(bundle);
        }
    }
}
