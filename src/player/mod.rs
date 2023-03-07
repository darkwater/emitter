use bevy::prelude::*;

mod startup;
mod systems;

#[derive(Component)]
pub struct PlayerShip;

#[derive(Component, Default)]
pub struct ShipEngine {
    pub target_velocity: Vec3,
    pub power: f32,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(startup::spawn_player)
            .add_system(systems::move_player_ship)
            .add_system(systems::apply_ship_engine)
            .add_system(systems::follow_player_ship);
    }
}
