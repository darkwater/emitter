use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::PrimitiveTopology,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "0443de5c-e5a4-4cba-a976-3912071cc8cb"]
pub struct Model {
    pub lines: Vec<(Vec3, Vec3)>,
    pub colors: ColorSpec,
}

#[derive(Serialize, Deserialize)]
pub enum ColorSpec {
    Uniform(Color),
    PerLine(Vec<Color>),
    PerVertex(Vec<(Color, Color)>),
}

pub struct ModelLoader;
impl AssetLoader for ModelLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let model: Model = ron::de::from_bytes(bytes)?;
            let mut mesh = Mesh::new(PrimitiveTopology::LineList);

            let colors: Vec<_> = match model.colors {
                ColorSpec::PerVertex(colors) => colors
                    .iter()
                    .flat_map(|(a, b)| [a.as_rgba_f32(), b.as_rgba_f32()])
                    .collect(),
                ColorSpec::PerLine(colors) => colors
                    .into_iter()
                    .flat_map(|c| [c.as_rgba_f32(), c.as_rgba_f32()])
                    .collect(),
                ColorSpec::Uniform(color) => vec![color.as_rgba_f32(); model.lines.len() * 2],
            };

            let vertices: Vec<_> = model.lines.into_iter().flat_map(|(a, b)| [a, b]).collect();

            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

            load_context.set_default_asset(LoadedAsset::new(mesh));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mdl.ron"]
    }
}
