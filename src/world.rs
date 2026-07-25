//! Physics reality layer.
//!
//! Everything visible — floor, terminal chassis, screen surface — exists as
//! entities in the Rapier world. The OS session is not a separate desktop
//! window; its display is bound to a physics entity.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Marker: this entity is the in-world screen that shows the OS session.
#[derive(Component)]
pub struct TerminalScreen;

/// Marker: chassis / bezel around the screen.
#[derive(Component)]
pub struct TerminalChassis;

pub fn spawn_physics_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Floor
    commands.spawn((
        Name::new("Floor"),
        Mesh3d(meshes.add(Cuboid::new(20.0, 0.2, 20.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.15, 0.16, 0.18))),
        Transform::from_xyz(0.0, -0.1, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(10.0, 0.1, 10.0),
    ));

    // Terminal chassis (physical object in the world)
    commands.spawn((
        Name::new("TerminalChassis"),
        TerminalChassis,
        Mesh3d(meshes.add(Cuboid::new(1.7, 1.15, 0.12))),
        MeshMaterial3d(materials.add(Color::srgb(0.08, 0.09, 0.1))),
        Transform::from_xyz(0.0, 1.15, -0.06),
        RigidBody::Fixed,
        Collider::cuboid(0.85, 0.575, 0.06),
    ));

    // Screen surface — physics entity that will hold the terminal texture
    commands.spawn((
        Name::new("TerminalScreen"),
        TerminalScreen,
        Transform::from_xyz(0.0, 1.2, 0.02),
        // Mesh/material added in setup_terminal_display once the Image exists.
        RigidBody::Fixed,
        Collider::cuboid(0.8, 0.5, 0.01),
    ));

    println!("[Physics] Reality layer online — terminal screen is a world entity");
}
