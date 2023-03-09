//! Create a custom material to draw basic lines in 3D

use std::f32::consts::PI;

use bevy::{
    core_pipeline::{
        bloom::BloomSettings, clear_color::ClearColorConfig, tonemapping::Tonemapping,
    },
    log::LogPlugin,
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
use bevy_hanabi::{prelude::*, EffectAsset};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;

use crate::{
    bullet::BulletPlugin, damageable::despawn_if_dead, player::PlayerPlugin,
    utils::zlock::ZLockPlugin, weapon::WeaponPlugin,
};

mod bullet;
mod collision_groups;
mod damageable;
mod player;
mod weapon;
mod utils {
    pub mod drawing;
    pub mod look_at_2d;
    pub mod zlock;
}

const CAMERA_OFFSET: Vec3 = Vec3::new(0., -5., 50.);

fn main() {
    println!("{}", std::env::current_dir().unwrap().display());

    App::new()
        .insert_resource(Msaa::Off)
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::INFO,
            filter: "emitter=trace,wgpu=warn".to_string(),
        }))
        .add_plugin(MaterialPlugin::<LineMaterial>::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(HanabiPlugin)
        .add_startup_system(setup)
        .add_startup_system(disable_gravity)
        .add_startup_system(setup_particles)
        .add_system(cycle_msaa)
        .add_system(toggle_debug_render)
        .add_system(despawn_if_dead)
        .add_plugin(PlayerPlugin)
        .add_plugin(WeaponPlugin)
        .add_plugin(BulletPlugin)
        // .add_plugin(ZLockPlugin)
        .run();
}

fn disable_gravity(mut conf: ResMut<RapierConfiguration>) {
    conf.gravity = Vec3::ZERO;
}

fn toggle_debug_render(
    mut debug_render: ResMut<DebugRenderContext>,
    input: Res<Input<MouseButton>>,
) {
    debug_render.enabled = input.pressed(MouseButton::Right);
}

fn setup_particles(mut effects: ResMut<Assets<EffectAsset>>, mut commands: Commands) {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(100.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(0.1, Vec4::new(50.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(0.9, Vec4::new(20.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(2.0, 0.0, 0.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(0.10));
    size_gradient1.add_key(0.3, Vec2::splat(0.05));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let effect1 = effects.add(
        EffectAsset {
            name: "firework".to_string(),
            capacity: 32768,
            spawner: Spawner::once(50.0.into(), false),
            ..Default::default()
        }
        .init(InitPositionSphereModifier {
            center: Vec3::ZERO,
            radius: 0.4,
            dimension: ShapeDimension::Volume,
        })
        .init(InitVelocitySphereModifier {
            center: Vec3::ZERO,
            // Give a bit of variation by randomizing the initial speed
            speed: Value::Uniform((1., 5.)),
        })
        .init(InitLifetimeModifier {
            // Give a bit of variation by randomizing the lifetime per particle
            lifetime: Value::Uniform((0.2, 0.8)),
        })
        .init(InitAgeModifier {
            // Give a bit of variation by randomizing the age per particle. This will control the
            // starting color and starting size of particles.
            age: Value::Uniform((0.0, 0.07)),
        })
        // .update(LinearDragModifier { drag: 5. })
        // .update(AccelModifier::constant(Vec3::new(0., -8., 0.)))
        .render(ColorOverLifetimeModifier { gradient: color_gradient1 })
        .render(SizeOverLifetimeModifier { gradient: size_gradient1 }),
    );

    commands.spawn((Name::new("firework"), ParticleEffectBundle {
        effect: ParticleEffect::new(effect1),
        transform: Transform::IDENTITY,
        ..Default::default()
    }));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
    mut window: Query<&mut Window>,
) {
    let mut window = window.single_mut();
    window.cursor.icon = CursorIcon::Crosshair;
    window.cursor.visible = false;

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

    let collider = Collider::trimesh(
        vec![
            Vec3::Z,
            -Vec3::Z,
            Vec3::X * 10. + Vec3::Z,
            Vec3::X * 10. + -Vec3::Z,
            Vec3::Y * 10. + Vec3::Z,
            Vec3::Y * 10. + -Vec3::Z,
        ],
        vec![[0, 2, 1], [1, 2, 3], [4, 0, 5], [5, 0, 1]],
    );

    commands.spawn((
        MaterialMeshBundle {
            mesh: triangle.clone(),
            transform: Transform::from_rotation(Quat::from_rotation_z(PI * 0.))
                .with_translation(Vec3::X * 10. + Vec3::Y * 10.),
            material: materials.add(LineMaterial { color: Color::CYAN * 4. }),
            ..default()
        },
        RigidBody::Fixed,
        collider.clone(),
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: triangle.clone(),
            transform: Transform::from_rotation(Quat::from_rotation_z(PI * 0.5))
                .with_translation(Vec3::X * -10. + Vec3::Y * 10.),
            material: materials.add(LineMaterial { color: Color::CYAN * 4. }),
            ..default()
        },
        RigidBody::Fixed,
        collider.clone(),
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: triangle.clone(),
            transform: Transform::from_rotation(Quat::from_rotation_z(PI * 1.))
                .with_translation(Vec3::X * -10. + Vec3::Y * -10.),
            material: materials.add(LineMaterial { color: Color::CYAN * 4. }),
            ..default()
        },
        RigidBody::Fixed,
        collider.clone(),
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: triangle.clone(),
            transform: Transform::from_rotation(Quat::from_rotation_z(PI * 1.5))
                .with_translation(Vec3::X * 10. + Vec3::Y * -10.),
            material: materials.add(LineMaterial { color: Color::CYAN * 4. }),
            ..default()
        },
        RigidBody::Fixed,
        collider.clone(),
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: triangle.clone(),
            transform: Transform::from_rotation(Quat::from_rotation_z(PI * 1.25))
                .with_translation(Vec3::Y * 40.)
                .with_scale(Vec3::new(3., 3., 1.5)),
            material: materials.add(LineMaterial { color: Color::BLUE * 4. }),
            ..default()
        },
        RigidBody::Fixed,
        collider.clone(),
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: triangle.clone(),
            transform: Transform::from_rotation(Quat::from_rotation_z(PI * 1.75))
                .with_translation(Vec3::X * -40.)
                .with_scale(Vec3::new(3., 3., 1.5)),
            material: materials.add(LineMaterial { color: Color::BLUE * 4. }),
            ..default()
        },
        RigidBody::Fixed,
        collider.clone(),
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: triangle.clone(),
            transform: Transform::from_rotation(Quat::from_rotation_z(PI * 0.25))
                .with_translation(Vec3::Y * -40.)
                .with_scale(Vec3::new(3., 3., 1.5)),
            material: materials.add(LineMaterial { color: Color::BLUE * 4. }),
            ..default()
        },
        RigidBody::Fixed,
        collider.clone(),
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: triangle,
            transform: Transform::from_rotation(Quat::from_rotation_z(PI * 0.75))
                .with_translation(Vec3::X * 40.)
                .with_scale(Vec3::new(3., 3., 1.5)),
            material: materials.add(LineMaterial { color: Color::BLUE * 4. }),
            ..default()
        },
        RigidBody::Fixed,
        collider,
    ));

    // camera
    commands.spawn((
        Name::new("Camera"),
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
