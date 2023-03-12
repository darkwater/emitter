use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use big_brain::prelude::*;

use crate::{
    collision_groups,
    damageable::Damageable,
    player::PlayerShip,
    team::Team,
    utils::{drawing::circle, zlock::ZLocked},
    LineList, LineMaterial,
};

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
            .add_system(chase_scorer_system)
            .add_system(chase_action_system)
            .add_system(contact_damage_system)
            .register_type::<ContactDamage>()
            .register_type::<PlayerChaser>();
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    for _ in 0..20 {
        commands.spawn((
            Name::new("Enemy"),
            Enemy,
            MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(LineList { lines: circle(0.5, 6) })),
                transform: Transform::from_xyz(-18., -18., 0.),
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

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PlayerChaser {
    pub max_proximity: f32,
    pub los: bool,
}

#[derive(Component, ScorerBuilder, Debug, Clone)]
pub struct Chase;

#[derive(Component, ActionBuilder, Debug, Clone)]
pub struct Chasing;

fn chase_scorer_system(
    players: Query<&Transform, With<PlayerShip>>,
    enemies: Query<(&Transform, &PlayerChaser)>,
    mut query: Query<(&Actor, &mut Score, &ScorerSpan), With<Chase>>,
) {
    for (Actor(actor), mut score, _span) in &mut query {
        if let Ok((transform, chaser)) = enemies.get(*actor) {
            for player in &mut players.iter() {
                let player_distance = player.translation.distance_squared(transform.translation);

                if player_distance < chaser.max_proximity.powi(2) {
                    score.set(1.);
                }
            }
        }
    }
}

fn chase_action_system(
    time: Res<Time>,
    mut enemies: Query<(&Transform, &mut ExternalImpulse), Without<PlayerShip>>,
    players: Query<(&Transform, &PlayerShip)>,
    mut query: Query<(&Actor, &mut ActionState, &ActionSpan, &Chasing)>,
) {
    for (Actor(actor), mut state, span, _) in &mut query {
        let _guard = span.span().enter();

        if let Ok((transform, mut impulse)) = enemies.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    debug!("start chasing!");
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    let player = players.iter().next().unwrap();

                    let direction = (player.0.translation - transform.translation).normalize();

                    impulse.impulse += direction * 10. * time.delta_seconds();
                }
                ActionState::Cancelled => {
                    debug!("chase cancelled");
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }
}

fn contact_damage_system(
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
                        if source_team.can_damage(target_team) {
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

// fn contact_damage_system(
//     context: Res<RapierContext>,
//     sources: Query<(Entity, &ContactDamage), Without<PlayerShip>>,
//     mut targets: Query<(&RigidBody, &mut Damageable), With<PlayerShip>>,
// ) {
//     for pair in context.contact_pairs() {
//         if !pair.has_any_active_contacts() {
//             continue;
//         }

//         // let (damageable, contact_damage) = query_intersection();
//     }
// }

// fn query_intersection<
//     'a,
//     'b,
//     'c: 'a,
//     'd: 'b,
//     Q1: WorldQuery,
//     Q2: WorldQuery,
//     F1: ReadOnlyWorldQuery,
//     F2: ReadOnlyWorldQuery,
// >(
//     (c1, c2): (Entity, Entity),
//     (q1, q2): (&'c mut Query<Q1, F1>, &'d mut Query<Q2, F2>),
// ) -> Option<(Q1::Item<'a>, Q2::Item<'b>)> {
//     // if let Ok(e1) = q1.get_mut(c1) {
//     //     if let Ok(e2) = q2.get_mut(c2) {
//     //         return Some((e1, e2));
//     //     }
//     // }

//     // if let Ok(e1) = q1.get_mut(c2) {
//     //     if let Ok(e2) = q2.get_mut(c1) {
//     //         return Some((e1, e2));
//     //     }
//     // }

//     {
//         let e1 = q1.get_mut(c1);
//         let e2 = q2.get_mut(c2);

//         if e1.is_ok() && e2.is_ok() {
//             return Some((e1.unwrap(), e2.unwrap()));
//         }
//     }

//     {
//         let e1 = q1.get_mut(c2);
//         let e2 = q2.get_mut(c1);

//         if let (Ok(e1), Ok(e2)) = (e1, e2) {
//             return Some((e1, e2));
//         }
//     }

//     None
// }
