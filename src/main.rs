//! Create a custom material to draw basic lines in 3D

use std::f32::consts::PI;

use bevy::{
    core_pipeline::{
        bloom::BloomSettings, clear_color::ClearColorConfig, tonemapping::Tonemapping,
    },
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};

use crate::player::PlayerPlugin;

mod player;
mod utils {
    pub mod drawing;
}

const CAMERA_OFFSET: Vec3 = Vec3::new(0., -5., 50.);

fn main() {
    println!("{}", std::env::current_dir().unwrap().display());

    App::new()
        .insert_resource(Msaa::Off)
        .add_plugins(DefaultPlugins)
        .add_plugin(MaterialPlugin::<LineMaterial>::default())
        .add_startup_system(setup)
        .add_system(cycle_msaa)
        .add_system(apply_inertia)
        .add_plugin(PlayerPlugin)
        .run();
}

#[derive(Component, Default)]
pub struct Inertia {
    pub velocity: Vec3,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
    mut window: Query<&mut Window>,
) {
    let mut window = window.single_mut();
    window.cursor.icon = CursorIcon::Crosshair;

    // triangle
    let triangle = meshes.add(Mesh::from(LineStrip {
        points: vec![
            Vec3::ZERO,
            Vec3::X * 10.,
            Vec3::Z * 5.,
            Vec3::Y * 10.,
            Vec3::ZERO,
            Vec3::Z * 5.,
        ],
    }));

    commands.spawn(MaterialMeshBundle {
        mesh: triangle.clone(),
        transform: Transform::from_rotation(Quat::from_rotation_z(PI * 0.))
            .with_translation(Vec3::X * -15. + Vec3::Y * -15.),
        material: materials.add(LineMaterial { color: Color::CYAN * 4. }),
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        mesh: triangle.clone(),
        transform: Transform::from_rotation(Quat::from_rotation_z(PI * 0.5))
            .with_translation(Vec3::X * 15. + Vec3::Y * -15.),
        material: materials.add(LineMaterial { color: Color::CYAN * 4. }),
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        mesh: triangle.clone(),
        transform: Transform::from_rotation(Quat::from_rotation_z(PI * 1.))
            .with_translation(Vec3::X * 15. + Vec3::Y * 15.),
        material: materials.add(LineMaterial { color: Color::CYAN * 4. }),
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        mesh: triangle,
        transform: Transform::from_rotation(Quat::from_rotation_z(PI * 1.5))
            .with_translation(Vec3::X * -15. + Vec3::Y * 15.),
        material: materials.add(LineMaterial { color: Color::CYAN * 4. }),
        ..default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            camera: Camera { hdr: true, ..default() },
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_translation(CAMERA_OFFSET).looking_at(Vec3::ZERO, Vec3::Z),
            ..default()
        },
        BloomSettings::default(),
        Inertia::default(),
    ));
}

fn cycle_msaa(input: Res<Input<KeyCode>>, mut msaa: ResMut<Msaa>) {
    if input.just_pressed(KeyCode::Key1) {
        info!("Not using MSAA");
        *msaa = Msaa::Off;
    }

    if input.just_pressed(KeyCode::Key2) {
        info!("Using 2x MSAA");
        *msaa = Msaa::Sample2;
    }

    if input.just_pressed(KeyCode::Key4) {
        info!("Using 4x MSAA");
        *msaa = Msaa::Sample4;
    }

    if input.just_pressed(KeyCode::Key8) {
        info!("Using 8x MSAA");
        *msaa = Msaa::Sample8;
    }
}

fn apply_inertia(mut query: Query<(&mut Transform, &Inertia)>, time: Res<Time>) {
    for (mut transform, inertia) in query.iter_mut() {
        transform.translation += inertia.velocity * time.delta_seconds();
    }
}

#[derive(Default, AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "050ce6ac-080a-4d8c-b6b5-b5bab7560d8f"]
pub struct LineMaterial {
    #[uniform(0)]
    color: Color,
}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/line_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}

/// A list of lines with a start and end position
#[derive(Debug, Clone)]
pub struct LineList {
    pub lines: Vec<(Vec3, Vec3)>,
}

impl From<LineList> for Mesh {
    fn from(line: LineList) -> Self {
        // This tells wgpu that the positions are list of lines
        // where every pair is a start and end point
        let mut mesh = Mesh::new(PrimitiveTopology::LineList);

        let vertices: Vec<_> = line.lines.into_iter().flat_map(|(a, b)| [a, b]).collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh
    }
}

/// A list of points that will have a line drawn between each consecutive points
#[derive(Debug, Clone)]
pub struct LineStrip {
    pub points: Vec<Vec3>,
}

impl From<LineStrip> for Mesh {
    fn from(line: LineStrip) -> Self {
        // This tells wgpu that the positions are a list of points
        // where a line will be drawn between each consecutive point
        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, line.points);
        mesh
    }
}
