//! Real Termux (Android) environment inside a physics-engine world.
//!
//! Physics is the reality layer. Android hosts Termux; the framebuffer is
//! shown on a mesh entity that exists in the Rapier world.

mod android_bridge;
mod display;
mod framebuffer;
mod world;

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use android_bridge::{AndroidBridge, AndroidPlugin};
use display::setup_screen_material;
use framebuffer::{FramebufferPlugin, FramebufferState};
use world::{spawn_physics_world, TerminalScreen};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Termux Inside Physics World".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(AndroidPlugin)
        .add_plugins(FramebufferPlugin)
        .add_systems(
            Startup,
            (setup_camera_light, spawn_physics_world, setup_screen_material).chain(),
        )
        .add_systems(Update, (forward_input_to_android, apply_framebuffer_to_screen))
        .run();
}

fn setup_camera_light(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 1.35, 2.8).looking_at(Vec3::new(0.0, 1.15, 0.0), Vec3::Y),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.65, 0.7, 0.75),
        brightness: 0.4,
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 14_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(2.5, 5.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn forward_input_to_android(
    keys: Res<ButtonInput<KeyCode>>,
    mut events: EventReader<KeyboardInput>,
    bridge: Res<AndroidBridge>,
) {
    if keys.just_pressed(KeyCode::KeyQ) && keys.pressed(KeyCode::ControlLeft) {
        std::process::exit(0);
    }
    if keys.just_pressed(KeyCode::KeyT) && keys.pressed(KeyCode::ControlLeft) {
        bridge.launch_termux();
        return;
    }

    for ev in events.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        match &ev.logical_key {
            Key::Character(c) => bridge.inject_text(c),
            Key::Space => bridge.inject_text(" "),
            Key::Enter => bridge.inject_keyevent(66), // KEYCODE_ENTER
            Key::Backspace => bridge.inject_keyevent(67), // KEYCODE_DEL
            Key::Tab => bridge.inject_keyevent(61),
            Key::Escape => bridge.inject_keyevent(111),
            Key::ArrowUp => bridge.inject_keyevent(19),
            Key::ArrowDown => bridge.inject_keyevent(20),
            Key::ArrowLeft => bridge.inject_keyevent(21),
            Key::ArrowRight => bridge.inject_keyevent(22),
            _ => {}
        }
    }
}

fn apply_framebuffer_to_screen(
    fb: Res<FramebufferState>,
    mut images: ResMut<Assets<Image>>,
    query: Query<&display::ScreenSurface, With<TerminalScreen>>,
) {
    let Some(frame) = fb.latest_frame() else {
        return;
    };
    for surface in query.iter() {
        if let Some(image) = images.get_mut(&surface.image) {
            display::blit_frame_into_image(image, &frame);
        }
    }
}
