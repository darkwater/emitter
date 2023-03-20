#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use assets::AssetsPlugin;
use bevy::{
    core_pipeline::{
        bloom::BloomSettings, clear_color::ClearColorConfig, tonemapping::Tonemapping,
    },
    log::LogPlugin,
    prelude::*,
    window::{Cursor, PresentMode, WindowFocused, WindowMode, WindowResolution},
};
use bevy_hanabi::prelude::*;
use bevy_rapier3d::prelude::*;
use big_brain::BigBrainPlugin;
use editor::{input::EditorAction, EditorWindow};
use leafwing_input_manager::prelude::{InputManagerPlugin, ToggleActions};

use crate::{
    bullet::BulletPlugin,
    damageable::despawn_if_dead,
    editor::EditorPlugin,
    enemy::EnemyPlugin,
    line_material::LineMaterial,
    player::{input::PlayerAction, systems::PlayerFollower, PlayerPlugin},
    utils::zlock::ZLockPlugin,
    weapon::WeaponPlugin,
};

mod assets;
mod bullet;
mod collision_groups;
mod damageable;
mod editor;
mod enemy;
mod line_material;
mod player;
mod render_layers;
mod team;
mod weapon;
mod utils {
    pub mod drawing;
    pub mod look_at_2d;
    pub mod zlock;
}

const CAMERA_OFFSET: Vec3 = Vec3::new(0., -5., 50.);

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    level: bevy::log::Level::INFO,
                    filter: "emitter=trace,wgpu=warn,big_brain=debug".to_string(),
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        cursor: {
                            let mut cursor = Cursor::default();
                            cursor.icon = CursorIcon::Crosshair;
                            cursor
                        },
                        present_mode: PresentMode::AutoNoVsync,
                        mode: WindowMode::Windowed,
                        resolution: WindowResolution::new(1920., 1080.),
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        title: "Emitter".to_owned(),
                        transparent: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin { watch_for_changes: true, ..default() }),
        )
        .add_plugin(MaterialPlugin::<LineMaterial>::default())
        .register_type::<LineMaterial>()
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin { enabled: false, ..default() })
        .add_plugin(HanabiPlugin)
        .add_plugin(BigBrainPlugin)
        .add_plugin(InputManagerPlugin::<PlayerAction>::default())
        .add_plugin(InputManagerPlugin::<EditorAction>::default())
        .add_plugin(bevy_egui::EguiPlugin)
        .insert_resource(ToggleActions::<PlayerAction>::DISABLED)
        .insert_resource(ToggleActions::<EditorAction>::DISABLED)
        .add_startup_system(setup_windows_cameras)
        .add_startup_system(disable_gravity)
        .add_startup_system(load_scene)
        // .add_startup_system(map::spawn_map)
        .add_system(cycle_msaa)
        .add_system(despawn_if_dead)
        .add_system(handle_window_focus_events)
        .add_system(replace_standard_material)
        .add_plugin(AssetsPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(WeaponPlugin)
        .add_plugin(BulletPlugin)
        .add_plugin(EnemyPlugin)
        .add_plugin(ZLockPlugin)
        .add_plugin(EditorPlugin)
        .run();
}

fn replace_standard_material(
    mut query: Query<Entity, With<Handle<StandardMaterial>>>,
    mut commands: Commands,
    mut line_materials: ResMut<Assets<LineMaterial>>,
) {
    for entity in query.iter_mut() {
        println!("replacing");
        commands
            .entity(entity)
            .remove::<Handle<StandardMaterial>>()
            .insert(line_materials.add(LineMaterial::default()));
    }
}

fn disable_gravity(mut conf: ResMut<RapierConfiguration>) {
    conf.gravity = Vec3::ZERO;
}

fn setup_windows_cameras(mut commands: Commands, mut windows: Query<Entity, With<Window>>) {
    let window = windows.single_mut();
    commands.entity(window).insert(PlayerWindow);

    commands.spawn((
        Name::new("Camera"),
        Camera3dBundle {
            camera: Camera { hdr: true, ..default() },
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::rgba(0., 0., 0., 0.)),
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_translation(CAMERA_OFFSET).looking_at(Vec3::ZERO, Vec3::Z),
            ..default()
        },
        BloomSettings::default(),
        PlayerFollower,
    ));
}

fn load_scene(
    commands: Commands,
    materials: ResMut<Assets<LineMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
) {
    editor::scene::load_scene("assets/maps/world.map.ron", commands, materials, meshes);
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

#[derive(Component)]
pub struct PlayerWindow;

#[derive(Component)]
pub struct FocusedWindow;

fn handle_window_focus_events(
    mut commands: Commands,
    mut events: EventReader<WindowFocused>,
    windows: Query<AnyOf<(&PlayerWindow, &EditorWindow)>, With<Window>>,
    mut player_input: ResMut<ToggleActions<PlayerAction>>,
    mut editor_input: ResMut<ToggleActions<EditorAction>>,
) {
    for event in events.iter() {
        if event.focused {
            commands.entity(event.window).insert(FocusedWindow);
        } else {
            commands.entity(event.window).remove::<FocusedWindow>();
        }

        let (player, editor) = windows.get(event.window).unwrap();

        if player.is_some() {
            player_input.enabled = event.focused;
        }

        if editor.is_some() {
            editor_input.enabled = event.focused;
        }
    }
}
