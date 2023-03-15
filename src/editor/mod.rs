use std::f32::consts::PI;

use bevy::{
    core_pipeline::{
        bloom::BloomSettings, clear_color::ClearColorConfig, tonemapping::Tonemapping,
    },
    input::mouse::MouseMotion,
    math::DVec2,
    prelude::*,
    render::camera::RenderTarget,
    window::{CursorGrabMode, PresentMode, WindowMode, WindowRef, WindowResolution},
};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use leafwing_input_manager::{prelude::*, InputManagerBundle};

use self::input::EditorAction;

pub mod input;
pub mod ui;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(DefaultInspectorConfigPlugin)
            .add_plugin(ui::EditorUiPlugin)
            .init_resource::<ui::UiState>()
            // .init_resource::<EditorCameraOffset>()
            .add_startup_system(spawn_window)
            .add_system(camera_follow_focus)
            .add_system(move_camera_focus)
            .add_system(grab_cursor_on_move);
    }
}

#[derive(Component)]
pub struct EditorWindow;

#[derive(Component)]
pub struct EditorCamera;

#[derive(Component)]
pub struct EditorCameraFocus {
    distance: f32,
}

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
        Name::new("Editor Camera"),
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
            transform: Transform::from_xyz(0., 0.1, 75.).looking_at(Vec3::ZERO, Vec3::Z),
            ..default()
        },
        BloomSettings::default(),
        EditorCamera,
    ));

    commands.spawn((
        Name::new("Editor Camera Focus"),
        Transform::from_xyz(0., 0., 0.).with_rotation(Quat::from_rotation_z(PI)),
        EditorCameraFocus { distance: 75. },
        InputManagerBundle::<EditorAction> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(MouseButton::Right, EditorAction::Rotate)
                .insert(
                    VirtualDPad {
                        up: KeyCode::W.into(),
                        down: KeyCode::S.into(),
                        left: KeyCode::A.into(),
                        right: KeyCode::D.into(),
                    },
                    EditorAction::Move,
                )
                .build(),
        },
    ));
}

fn camera_follow_focus(
    focus: Query<(&Transform, &EditorCameraFocus), Without<EditorCamera>>,
    // offset: Res<EditorCameraOffset>,
    mut camera: Query<&mut Transform, With<EditorCamera>>,
) {
    let (focus_transform, focus) = focus.single();

    for mut camera_transform in camera.iter_mut() {
        *camera_transform =
            focus_transform.mul_transform(Transform::from_xyz(0., 0., focus.distance));

        // camera_transform.look_at(focus_transform.translation, Vec3::Z);
    }
}

fn move_camera_focus(
    mut query: Query<(&mut Transform, &ActionState<EditorAction>), With<EditorCameraFocus>>,
    // mut offset: ResMut<EditorCameraOffset>,
    mut mouse_events: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    for (mut transform, action_state) in query.iter_mut() {
        let up = transform.up();
        let right = transform.right();

        if action_state.pressed(EditorAction::Move) {
            let axis_pair = action_state.clamped_axis_pair(EditorAction::Move).unwrap();
            let vec = axis_pair.xy();
            transform.translation += right * vec.x * time.delta_seconds() * 20.;

            let up = Vec3::new(up.x, up.y, 0.).normalize();
            transform.translation += up * vec.y * time.delta_seconds() * 20.;
        }

        if action_state.pressed(EditorAction::Rotate) {
            for event in mouse_events.iter() {
                let delta = event.delta;
                let vec = Vec2::new(delta.x, delta.y);

                transform.rotate(Quat::from_rotation_z(vec.x * time.delta_seconds() * -0.5));
                transform.rotate(Quat::from_axis_angle(right, vec.y * time.delta_seconds() * -0.5));
            }
        }
    }
}

fn grab_cursor_on_move(
    query: Query<&ActionState<EditorAction>>,
    mut window: Query<&mut Window, With<EditorWindow>>,
    camera: Query<&mut Camera, With<EditorCamera>>,
) {
    let mut window = window.single_mut();
    let action_state = query.single();

    if action_state.just_pressed(EditorAction::Rotate) {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    } else if action_state.just_released(EditorAction::Rotate) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;

        let window_height = window.physical_height() as f64;
        window.set_physical_cursor_position(
            camera
                .single()
                .viewport
                .as_ref()
                .map(|v| v.physical_position + v.physical_size / 2)
                .map(|v| DVec2::new(v.x as f64, v.y as f64 * -1. + window_height)),
        );
    }
}
