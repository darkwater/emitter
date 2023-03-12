use bevy::prelude::*;

#[derive(Component, Reflect, Default, PartialEq, Eq, Clone, Copy, Debug)]
#[reflect(Component)]
pub enum Team {
    #[default]
    None,
    Player,
    Enemy,
}

impl Team {
    pub fn can_damage(&self, other: &Team) -> bool {
        if *self == Team::None || *other == Team::None {
            return true;
        }

        self != other
    }
}
