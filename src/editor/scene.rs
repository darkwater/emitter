use std::{fs::File, io::Write, path::Path};

use bevy::{prelude::*, tasks::IoTaskPool};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::line_material::{LineList, LineMaterial};

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SaveScene>().add_system(save_scene);
    }
}

pub struct SaveScene;

// #[derive(Reflect, Component, Default)]
// #[reflect(Component)]
// pub struct Saveable;

// pub fn save_scene(
//     world: &World,
//     mut event: EventReader<SaveScene>,
//     query: Query<(&Transform, &Handle<LineMaterial>, &Handle<Mesh>)>,
//     materials: Res<Assets<LineMaterial>>,
//     meshes: Res<Assets<Mesh>>,
// ) {
//     if event.iter().next().is_none() {
//         return;
//     }

//     println!("Saving scene");

//     // let mut scene_world = World::new();

//     // for (transform, material, mesh) in query.iter() {
//     //     scene_world.spawn((*transform, material.clone(), mesh.clone()));
//     // }

//     // The TypeRegistry resource contains information about all registered types (including
//     // components). This is used to construct scenes.
//     let type_registry = world.resource::<AppTypeRegistry>();
//     let mut scene = DynamicScene::from_world(world, type_registry);

//     let type_reg = type_registry.internal.read();
//     scene.entities.retain_mut(|ent| {
//         println!("Components: {}", ent.components.len());

//         ent.components
//             .retain(|comp| type_reg.get(comp.type_id()).is_some());

//         println!("Remaining components: {:?}", ent.components);

//         !ent.components.is_empty()
//     });

//     // Scenes can be serialized like this:
//     let serialized_scene = scene.serialize_ron(type_registry).unwrap();

//     // Showing the scene in the console
//     info!("{}", serialized_scene);

//     // Writing the scene to a new file. Using a task to avoid calling the filesystem APIs in a system
//     // as they are blocking
//     IoTaskPool::get()
//         .spawn(async move {
//             // Write the scene RON data to file
//             File::create("assets/scenes/world.scn.ron")
//                 .and_then(|mut file| file.write(serialized_scene.as_bytes()))
//                 .expect("Error while writing scene to file");
//         })
//         .detach();
// }

#[derive(Serialize, Deserialize)]
struct Scene {
    map_meshes: Vec<MapMesh>,
}

#[derive(Serialize, Deserialize)]
struct MapMesh {
    pub transform: Transform,
    pub lines: LineList,
    pub color: Color,
}

pub fn save_scene(
    mut event: EventReader<SaveScene>,
    query: Query<(&Transform, &Handle<LineMaterial>, &Handle<Mesh>)>,
    materials: Res<Assets<LineMaterial>>,
    meshes: Res<Assets<Mesh>>,
) {
    if event.iter().next().is_none() {
        return;
    }

    let mut map_meshes = vec![];

    for (transform, material, mesh) in query.iter() {
        let material = materials.get(material).unwrap();
        let mesh = meshes.get(mesh).unwrap();

        if let Some(attr) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            let lines = attr
                .as_float3()
                .unwrap()
                .iter()
                .tuples()
                .map(|(a, b)| (Vec3::from(*a), Vec3::from(*b)))
                .collect();

            map_meshes.push(MapMesh {
                transform: *transform,
                lines: LineList { lines },
                color: material.color,
            });
        }
    }

    let scene = Scene { map_meshes };

    IoTaskPool::get()
        .spawn(async move {
            // Write the scene RON data to file
            File::create("assets/maps/world.scn.ron")
                .and_then(|mut file| {
                    file.write(
                        ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default())
                            .unwrap()
                            .as_bytes(),
                    )
                })
                .expect("Error while writing scene to file");
        })
        .detach();
}

pub fn load_scene<P: AsRef<Path>>(
    path: P,
    mut commands: Commands,
    mut materials: ResMut<Assets<LineMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let file = File::open(path).unwrap();
    let scene: Scene = ron::de::from_reader(file).unwrap();

    for map_mesh in scene.map_meshes {
        let mesh = meshes.add(Mesh::from(map_mesh.lines));
        let material = materials.add(LineMaterial { color: map_mesh.color });

        commands.spawn(MaterialMeshBundle {
            mesh,
            material,
            transform: map_mesh.transform,
            ..Default::default()
        });
    }
}
