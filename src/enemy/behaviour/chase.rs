use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use big_brain::prelude::*;

use crate::player::PlayerShip;

pub struct PlayerChaserPlugin;

impl Plugin for PlayerChaserPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(chase_scorer_system)
            .add_system(chase_action_system)
            .register_type::<PlayerChaser>();
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

pub fn chase_scorer_system(
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

pub fn chase_action_system(
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
