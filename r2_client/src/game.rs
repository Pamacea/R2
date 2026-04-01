//! Logique de jeu côté client

use bevy::prelude::*;
use r2_core::{components::*, types::*};

/// Plugin de jeu
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, movement);
    }
}

/// Setup initial
fn setup(mut commands: Commands) {
    // Caméra 2D
    commands.spawn(Camera2dBundle::default());

    // Lumière ambiante
    commands.spawn(PointLight2dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        point_light: PointLight {
            intensity: 1000.0,
            color: Color::WHITE,
            ..default()
        },
        ..default()
    });

    // Joueur local (cube rouge temporaire)
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.8, 0.2, 0.2),
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
        LocalPlayer,
        Position::origin(),
        Rotation::new(0.0),
        Velocity::zero(),
        Stats::new(100.0, 50.0, 100.0),
        ShipData::fighter(),
        CombatState::new(),
    ));
}

/// Système de mouvement (keyboard)
fn movement(
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &ShipData), With<LocalPlayer>>,
) {
    let (mut velocity, ship) = query.single_mut();

    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        let normalized = direction.normalize();
        velocity.x = normalized.x * ship.base_speed;
        velocity.y = normalized.y * ship.base_speed;
    } else {
        *velocity = Velocity::zero();
    }
}
