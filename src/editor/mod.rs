use bevy::{
    core_pipeline::{
        bloom::BloomSettings, clear_color::ClearColorConfig, tonemapping::Tonemapping,
    },
    prelude::*,
    render::camera::RenderTarget,
    window::{PresentMode, WindowMode, WindowRef, WindowResolution},
};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;

mod ui;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(DefaultInspectorConfigPlugin)
            .add_plugin(ui::EditorUiPlugin)
            .insert_resource(ui::UiState::new())
            .add_startup_system(spawn_window);
    }
}

#[derive(Component)]
pub struct EditorWindow;

#[derive(Component)]
pub struct EditorCamera;

pub fn spawn_window(mut commands: Commands) {
    let editor_window = commands
        .spawn((
            Window {
                present_mode: PresentMode::AutoNoVsync,
                mode: WindowMode::Windowed,
                resolution: WindowResolution::new(1920., 1080.),
                position: WindowPosition::Centered(MonitorSelection::Index(0)),
                title: "Emitter Editor".to_owned(),
                transparent: false,
                ..default()
            },
            EditorWindow,
        ))
        .id();

    commands.spawn((
        Name::new("Camera"),
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                target: RenderTarget::Window(WindowRef::Entity(editor_window)),
                ..default()
            },
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::rgba(0., 0., 0., 0.)),
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_xyz(0., 0., 75.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        BloomSettings::default(),
        EditorCamera,
    ));
}
