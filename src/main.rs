#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use std::time::Duration;

use assets::AssetsPlugin;
use bevy::{
    asset::ChangeWatcher,
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
mod egui_style;
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
                            Cursor {
                                icon: CursorIcon::Crosshair,
                                ..default()
                            }
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
                .set(AssetPlugin {
                    watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
                    ..default()
                }),
        )
        .add_plugins(MaterialPlugin::<LineMaterial>::default())
        .register_type::<LineMaterial>()
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin { enabled: false, ..default() })
        .add_plugins(HanabiPlugin)
        .add_plugins(BigBrainPlugin::new(PreUpdate))
        .add_plugins(InputManagerPlugin::<PlayerAction>::default())
        .add_plugins(InputManagerPlugin::<EditorAction>::default())
        .add_plugins(bevy_egui::EguiPlugin)
        .insert_resource(ToggleActions::<PlayerAction>::DISABLED)
        .insert_resource(ToggleActions::<EditorAction>::DISABLED)
        .add_systems(Startup, setup_windows_cameras)
        .add_systems(Startup, disable_gravity)
        .add_systems(Startup, load_scene)
        .add_systems(Update, egui_style::set_egui_style)
        // .add_startup_system(map::spawn_map)
        .add_systems(Update, cycle_msaa)
        .add_systems(Update, despawn_if_dead)
        .add_systems(Update, handle_window_focus_events)
        .add_systems(Update, replace_standard_material)
        .add_plugins(AssetsPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(WeaponPlugin)
        .add_plugins(BulletPlugin)
        .add_plugins(EnemyPlugin)
        .add_plugins(ZLockPlugin)
        .add_plugins(EditorPlugin)
        .run();
}

fn replace_standard_material(
    mut query: Query<Entity, With<Handle<StandardMaterial>>>,
    mut commands: Commands,
    mut line_materials: ResMut<Assets<LineMaterial>>,
) {
    for entity in query.iter_mut() {
        println!("replacing with line material");
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

fn load_scene(commands: Commands, meshes: ResMut<Assets<Mesh>>) {
    editor::scene::load_scene("assets/maps/world.map.ron", commands, meshes);
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
