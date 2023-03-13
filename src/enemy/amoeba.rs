use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use big_brain::{prelude::Highest, thinker::Thinker};

use super::{
    behaviour::chase::{Chase, Chasing, PlayerChaser},
    ContactDamage, Enemy,
};
use crate::{
    collision_groups,
    damageable::Damageable,
    team::Team,
    utils::{drawing::circle, zlock::ZLocked},
    LineList, LineMaterial,
};

#[derive(Component)]
pub struct AmoebaSpawnToken;

pub fn spawn_amoeba(
    query: Query<(Entity, &Transform), With<AmoebaSpawnToken>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    for (entity, transform) in query.iter() {
        commands.entity(entity).despawn_recursive();

        commands.spawn((
            Name::new("Enemy"),
            Enemy,
            MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(LineList { lines: circle(0.5, 6) })),
                transform: *transform,
                material: materials.add(LineMaterial { color: Color::GREEN * 2.5 }),
                ..default()
            },
            CollisionGroups::new(collision_groups::ENEMY, collision_groups::ALL),
            ExternalImpulse::default(),
            RigidBody::Dynamic,
            Collider::ball(0.5),
            ZLocked { angular: false },
            PlayerChaser { max_proximity: 15., los: true },
            ContactDamage { damage: 1. },
            Damageable { health: 2., max_health: 2. },
            Thinker::build()
                .label("chase")
                .picker(Highest)
                .when(Chase, Chasing),
            Team::Enemy,
        ));
    }
}
