use bevy::prelude::*;

use self::{map::MapLoader, model::ModelLoader};

pub mod map;
pub mod model;

pub struct AssetsPlugin;
impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset_loader(MapLoader)
            .add_asset_loader(ModelLoader);
    }
}
