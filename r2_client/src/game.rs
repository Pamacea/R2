//! Logique de jeu côté client
//!
//! Ce module intègre:
//! - Le rendu des vaisseaux (triangles)
//! - La gestion des entrées (ZQSD / flèches)
//! - L'interpolation réseau
//! - La boucle de jeu principale

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use r2_core::components::*;
use r2_core::types::ShipClass;

use crate::input::InputPlugin;
use crate::interpolation::{InterpolationPlugin, interpolate_entities};
use crate::renderer::ShipRendererPlugin;

/// Plugin de jeu principal
pub struct GamePlugin;

impl GamePlugin {
    /// Crée le plugin avec une configuration réseau
    pub fn with_network_config(_network_config: crate::network::ClientConfig) -> Self {
        // TODO: Stocker la config pour l'utiliser dans setup
        Self
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InputPlugin,
            ShipRendererPlugin,
            InterpolationPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_movement,
                update_positions,
                handle_camera,
            ),
        )
        .add_systems(PostUpdate, interpolate_entities);
    }
}

/// Setup initial du jeu
fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    // Caméra 2D
    commands.spawn(Camera2d);

    // Fond d'écran (espace)
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(5000.0, 5000.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.02, 0.02, 0.05))),
        Transform::from_translation(bevy::math::Vec3::new(0.0, 0.0, -10.0)),
    ));

    // Quelques étoiles pour le décor
    for _ in 0..100 {
        let x = (rand::random::<f32>() - 0.5) * 4000.0;
        let y = (rand::random::<f32>() - 0.5) * 4000.0;
        let size = rand::random::<f32>() * 3.0 + 1.0;
        let brightness = rand::random::<f32>() * 0.5 + 0.3;

        commands.spawn((
            Mesh2d(meshes.add(Circle { radius: size })),
            MeshMaterial2d(materials.add(Color::srgba(
                brightness,
                brightness,
                brightness,
                1.0,
            ))),
            Transform::from_translation(bevy::math::Vec3::new(x, y, -5.0)),
        ));
    }

    // Joueur local (triangle rouge)
    spawn_player_ship(&mut commands, &mut meshes, &mut materials);

    info!("Game initialized - use ZQSD or Arrow keys to move");
}

/// Spawn le vaisseau du joueur
fn spawn_player_ship(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let size = 30.0;

    // Triangle pour le vaisseau
    let vertices: Vec<[f32; 3]> = vec![
        [0.0, size / 2.0, 0.0],
        [-size * 0.35, -size / 2.0, 0.0],
        [size * 0.35, -size / 2.0, 0.0],
    ];

    let mut ship_mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    ship_mesh.insert_attribute(bevy::render::mesh::Mesh::ATTRIBUTE_POSITION, vertices);

    commands.spawn((
        Mesh2d(meshes.add(ship_mesh)),
        MeshMaterial2d(materials.add(Color::srgb(0.2, 0.9, 0.3))), // Vert pour le joueur
        Transform::from_translation(bevy::math::Vec3::ZERO),
        GlobalTransform::default(),
        Visibility::default(),
        // Composants de jeu
        LocalPlayer,
        Position::origin(),
        Rotation::new(0.0),
        Velocity::zero(),
        Stats::new(100.0, 50.0, 100.0),
        ShipData::fighter(),
        CombatState::new(),
    ));
}

/// Système de mouvement du joueur (basé sur l'input)
fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Position, &mut Rotation, &mut Velocity, &ShipData), With<LocalPlayer>>,
) {
    let (mut pos, mut rot, mut vel, ship) = query.single_mut();

    let mut move_dir = bevy::math::Vec2::ZERO;

    // ZQSD (AZERTY) et flèches
    if keyboard.pressed(KeyCode::KeyZ) || keyboard.pressed(KeyCode::ArrowUp) {
        move_dir.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        move_dir.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyQ) || keyboard.pressed(KeyCode::ArrowLeft) {
        move_dir.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        move_dir.x += 1.0;
    }

    // Normaliser et appliquer la vitesse
    if move_dir != bevy::math::Vec2::ZERO {
        let normalized = move_dir.normalize();
        vel.x = normalized.x * ship.base_speed;
        vel.y = normalized.y * ship.base_speed;

        // Faire tourner le vaisseau vers la direction de mouvement
        let target_angle = normalized.y.atan2(normalized.x) - std::f32::consts::PI / 2.0;
        rot.angle = target_angle;
    } else {
        *vel = Velocity::zero();
    }

    // Mettre à jour la position
    let delta = time.delta().as_secs_f32();
    pos.x += vel.x * delta;
    pos.y += vel.y * delta;
}

/// Met à jour les positions des entités (d'après leur vélocité)
fn update_positions(
    mut query: Query<(&mut Transform, &Position, &Rotation), (With<ShipData>, Without<LocalPlayer>)>,
) {
    for (mut transform, pos, rot) in query.iter_mut() {
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
        transform.rotation = Quat::from_rotation_z(rot.angle);
    }
}

/// Fait suivre la caméra au joueur
fn handle_camera(
    mut camera_q: Query<&mut Transform, With<Camera2d>>,
    player_q: Query<&Position, With<LocalPlayer>>,
) {
    let Ok(mut camera_transform) = camera_q.get_single_mut() else {
        return;
    };

    let Ok(player_pos) = player_q.get_single() else {
        return;
    };

    camera_transform.translation.x = player_pos.x;
    camera_transform.translation.y = player_pos.y;
}

/// Système pour afficher des infos de debug
#[derive(Debug)]
pub struct DebugInfo {
    pub show_fps: bool,
    pub show_position: bool,
    pub show_network_stats: bool,
}

impl Default for DebugInfo {
    fn default() -> Self {
        Self {
            show_fps: true,
            show_position: true,
            show_network_stats: false,
        }
    }
}

/// Fonction utilitaire pour créer un vaisseau ennemi
pub fn spawn_enemy_ship(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: Position,
    ship_class: ShipClass,
) -> Entity {
    let size = match ship_class {
        ShipClass::Fighter => 30.0,
        ShipClass::Frigate => 45.0,
        ShipClass::Cruiser => 70.0,
        ShipClass::Hauler => 60.0,
    };

    let vertices: Vec<[f32; 3]> = vec![
        [0.0, size / 2.0, 0.0],
        [-size * 0.35, -size / 2.0, 0.0],
        [size * 0.35, -size / 2.0, 0.0],
    ];

    let mut ship_mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    ship_mesh.insert_attribute(bevy::render::mesh::Mesh::ATTRIBUTE_POSITION, vertices);

    commands
        .spawn((
            Mesh2d(meshes.add(ship_mesh)),
            MeshMaterial2d(materials.add(Color::srgb(0.9, 0.2, 0.2))), // Rouge pour ennemis
            Transform::from_translation(bevy::math::Vec3::new(position.x, position.y, 0.0)),
            Position { ..position },
            Rotation::new(0.0),
            Velocity::zero(),
            Stats::new(50.0, 25.0, 50.0),
            ShipData::fighter(),
            CombatState::new(),
        ))
        .id()
}
