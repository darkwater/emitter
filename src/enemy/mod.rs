use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use self::amoeba::AmoebaSpawnToken;
use crate::{damageable::Damageable, player::PlayerShip, team::Team};

pub mod amoeba;
pub mod behaviour {
    pub mod chase;
    // pub mod wander;
}

pub struct EnemyPlugin;

#[derive(Component)]
pub struct Enemy;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ContactDamage {
    pub damage: f32,
}

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(amoeba::spawn_amoeba)
            .add_system(contact_damage_system)
            .add_plugin(behaviour::chase::PlayerChaserPlugin)
            .register_type::<ContactDamage>();
    }
}

fn setup(mut commands: Commands) {
    for n in 0..20 {
        commands.spawn((Transform::from_xyz(-20. + n as f32, 0. - n as f32, 0.), AmoebaSpawnToken));
    }
}

pub fn contact_damage_system(
    context: Res<RapierContext>,
    sources: Query<(Entity, &ContactDamage, Option<&Team>), Without<PlayerShip>>,
    mut targets: Query<(&RigidBody, &mut Damageable, Option<&Team>), With<PlayerShip>>,
) {
    for (entity, cdamage, source_team) in sources.iter() {
        for contact_pair in context.contacts_with(entity) {
            let other_collider = if contact_pair.collider1() == entity {
                contact_pair.collider2()
            } else {
                contact_pair.collider1()
            };

            if let Ok((_, mut damageable, target_team)) = targets.get_mut(other_collider) {
                let mut damage = true;

                if let Some(source_team) = source_team {
                    if let Some(target_team) = target_team {
                        if !source_team.can_damage(target_team) {
                            damage = false;
                        }
                    }
                }

                if damage {
                    damageable.health -= cdamage.damage;
                }
            }
        }
    }
}
