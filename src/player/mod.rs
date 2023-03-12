use bevy::prelude::*;

mod startup;
mod systems;

#[derive(Component)]
pub struct PlayerShip;

#[derive(Component)]
pub struct PlayerAimTarget;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
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
            .add_system(systems::follow_player_ship)
            .add_system(systems::shoot)
            .add_system(systems::move_aim_target)
            .add_system(systems::aim_player_ship)
            .register_type::<ShipEngine>();
    }
}
