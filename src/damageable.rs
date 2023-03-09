use bevy::prelude::*;

#[derive(Component)]
pub struct Damageable {
    pub health: f32,
    pub max_health: f32,
}

pub fn despawn_if_dead(mut commands: Commands, query: Query<(Entity, &Damageable)>) {
    for (entity, damageable) in query.iter() {
        if damageable.health <= 0. {
            commands.entity(entity).despawn();
        }
    }
}
