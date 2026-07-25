//! Termux / real OS session inside a physics world.
//!
//! Physics (Rapier) is the reality layer. The terminal session is a real
//! process connected by a PTY; its output is drawn onto a surface that
//! exists as a physics entity in the world.

mod session;
mod display;
mod world;

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use display::{TerminalDisplay, update_terminal_texture};
use session::{SessionBridge, SessionPlugin, SessionOutput};
use world::{spawn_physics_world, TerminalScreen};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Termux in Physics World".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(SessionPlugin)
        .add_systems(
            Startup,
            (setup_camera_light, spawn_physics_world, setup_terminal_display).chain(),
        )
        .add_systems(
            Update,
            (
                forward_keyboard_to_session,
                drain_session_to_display,
                update_terminal_texture,
            ),
        )
        .run();
}

fn setup_camera_light(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 1.4, 3.2).looking_at(Vec3::new(0.0, 1.1, 0.0), Vec3::Y),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.7, 0.75, 0.8),
        brightness: 0.35,
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 12_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(2.0, 6.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn setup_terminal_display(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    screen_query: Query<Entity, With<TerminalScreen>>,
) {
    let display = TerminalDisplay::new(&mut images, 80, 24);
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(display.image_handle.clone()),
        unlit: true,
        ..default()
    });

    if let Ok(entity) = screen_query.get_single() {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Rectangle::new(1.6, 1.0))),
            MeshMaterial3d(material),
            display,
        ));
    } else {
        commands.spawn((
            Name::new("TerminalScreenFallback"),
            Mesh3d(meshes.add(Rectangle::new(1.6, 1.0))),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, 1.2, 0.0),
            display,
            TerminalScreen,
        ));
    }

    println!("[World] Terminal display surface created inside the physics world");
}

fn forward_keyboard_to_session(
    keys: Res<ButtonInput<KeyCode>>,
    mut text_events: EventReader<KeyboardInput>,
    mut bridge: ResMut<SessionBridge>,
) {
    for ev in text_events.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        match &ev.logical_key {
            Key::Character(c) => {
                let _ = bridge.write_str(c);
            }
            Key::Space => {
                let _ = bridge.write_str(" ");
            }
            Key::Enter => {
                let _ = bridge.write_str("\r");
            }
            Key::Backspace => {
                let _ = bridge.write_str("\x7f");
            }
            Key::Tab => {
                let _ = bridge.write_str("\t");
            }
            Key::Escape => {
                let _ = bridge.write_str("\x1b");
            }
            _ => {}
        }
    }

    // Hold Q to quit (Esc is useful inside the shell).
    if keys.just_pressed(KeyCode::KeyQ) && keys.pressed(KeyCode::ControlLeft) {
        std::process::exit(0);
    }
}

fn drain_session_to_display(
    mut output: ResMut<SessionOutput>,
    mut query: Query<&mut TerminalDisplay>,
) {
    let chunk = output.drain();
    if chunk.is_empty() {
        return;
    }
    for mut display in query.iter_mut() {
        display.feed_bytes(&chunk);
    }
}
