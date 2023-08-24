use bevy::prelude::*;

pub mod input;
pub mod startup;
pub mod systems;

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
        app.add_systems(Startup, startup::spawn_player)
            .add_systems(Update, systems::move_player_ship)
            .add_systems(Update, systems::apply_ship_engine)
            .add_systems(Update, systems::follow_player_ship)
            .add_systems(Update, systems::shoot)
            .add_systems(Update, systems::move_aim_target_gamepad)
            .add_systems(Update, systems::move_aim_target_mouse)
            .add_systems(Update, systems::aim_player_ship)
            .register_type::<ShipEngine>();
    }
}
