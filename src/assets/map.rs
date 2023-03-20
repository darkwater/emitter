use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::Transform,
    reflect::TypeUuid,
};
use serde::{Deserialize, Serialize};

use crate::line_material::LineList;

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "0443de5c-e5a4-4cba-a976-3912071cc8cb"]
pub struct Map {
    pub map_meshes: Vec<MapMesh>,
}

#[derive(Serialize, Deserialize)]
pub struct MapMesh {
    pub transform: Transform,
    pub lines: LineList,
}

pub struct MapLoader;
impl AssetLoader for MapLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let res: Map = ron::de::from_bytes(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(res));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["map.ron"]
    }
}
