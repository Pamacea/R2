//! Module de rendu pour les vaisseaux spatiaux
//!
//! Les vaisseaux sont rendus comme des triangles avec des couleurs
//! différentes selon la faction et le type.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use r2_core::{components::*, types::*};

/// Configuration du rendu
#[derive(Resource, Clone, Debug)]
pub struct RendererConfig {
    /// Échelle des vaisseaux
    pub ship_scale: f32,
    /// Activer/désactiver les ombres
    pub shadows_enabled: bool,
    /// Afficher les hitboxes (debug)
    pub show_hitboxes: bool,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            ship_scale: 1.0,
            shadows_enabled: false,
            show_hitboxes: false,
        }
    }
}

/// Plugin de rendu des vaisseaux
pub struct ShipRendererPlugin;

impl Plugin for ShipRendererPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RendererConfig>()
            .add_systems(Startup, setup_ship_assets)
            .add_systems(Update, update_ship_transforms);
    }
}

/// Ressource contenant les matériaux pour les vaisseaux
#[derive(Resource, Clone, Debug)]
pub struct ShipAssets {
    /// Matériau par défaut
    pub default_material: Handle<ColorMaterial>,
    /// Matériaux par faction
    pub faction_materials: std::collections::HashMap<Faction, Handle<ColorMaterial>>,
    /// Matériau pour le joueur local
    pub player_material: Handle<ColorMaterial>,
    /// Matériau pour les hostiles
    pub hostile_material: Handle<ColorMaterial>,
}

/// Setup des assets de vaisseaux
fn setup_ship_assets(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let default_material = materials.add(Color::WHITE);

    let mut faction_materials = std::collections::HashMap::new();

    // Couleurs par faction
    faction_materials.insert(Faction::Neutral, materials.add(Color::srgb(0.7, 0.7, 0.7)));
    faction_materials.insert(Faction::Terran, materials.add(Color::srgb(0.2, 0.4, 0.8)));
    faction_materials.insert(Faction::Martian, materials.add(Color::srgb(0.8, 0.2, 0.2)));
    faction_materials.insert(Faction::Beltalian, materials.add(Color::srgb(0.8, 0.6, 0.2)));
    faction_materials.insert(Faction::Pirate, materials.add(Color::srgb(0.6, 0.2, 0.6)));
    faction_materials.insert(Faction::Corporation, materials.add(Color::srgb(0.2, 0.7, 0.5)));

    let player_material = materials.add(Color::srgb(0.2, 0.9, 0.3));
    let hostile_material = materials.add(Color::srgb(0.9, 0.2, 0.2));

    commands.insert_resource(ShipAssets {
        default_material: default_material.clone(),
        faction_materials,
        player_material,
        hostile_material,
    });
}

/// Composant pour le rendu d'un vaisseau
#[derive(Component, Clone, Debug)]
pub struct ShipRender {
    /// Classe du vaisseau
    pub ship_class: ShipClass,
    /// Faction
    pub faction: Faction,
    /// Type d'entité
    pub entity_type: EntityType,
    /// Échelle personnalisée
    pub scale: f32,
}

impl ShipRender {
    /// Crée un rendu de vaisseau
    pub fn new(ship_class: ShipClass, faction: Faction, entity_type: EntityType) -> Self {
        let scale = match ship_class {
            ShipClass::Fighter => 1.0,
            ShipClass::Frigate => 1.5,
            ShipClass::Cruiser => 2.5,
            ShipClass::Hauler => 2.0,
        };

        Self {
            ship_class,
            faction,
            entity_type,
            scale,
        }
    }

    /// Retourne la taille du vaisseau en pixels
    pub fn size(&self) -> f32 {
        let base = match self.ship_class {
            ShipClass::Fighter => 30.0,
            ShipClass::Frigate => 45.0,
            ShipClass::Cruiser => 70.0,
            ShipClass::Hauler => 60.0,
        };
        base * self.scale
    }
}

/// Composant marquant le vaisseau comme sélectionné/ciblé
#[derive(Component, Clone, Debug)]
pub struct Targeted {
    /// Opacité de l'indicateur (0.0 à 1.0)
    pub alpha: f32,
}

impl Default for Targeted {
    fn default() -> Self {
        Self { alpha: 0.5 }
    }
}

/// Met à jour les transforms des vaisseaux
fn update_ship_transforms(
    mut query: Query<(&mut Transform, &Position, &Rotation), With<ShipRender>>,
) {
    for (mut transform, pos, rot) in query.iter_mut() {
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
        transform.rotation = Quat::from_rotation_z(rot.angle);
    }
}

/// Spawn un vaisseau avec tous ses composants
#[allow(clippy::too_many_arguments)]
pub fn spawn_ship_full(
    commands: &mut Commands,
    meshes: &mut Mut<Assets<Mesh>>,
    assets: &ShipAssets,
    network_id: u64,
    position: Position,
    rotation: Rotation,
    ship_class: ShipClass,
    faction: Faction,
    entity_type: EntityType,
    stats: Stats,
    is_local: bool,
) -> Entity {
    let size = match ship_class {
        ShipClass::Fighter => 30.0,
        ShipClass::Frigate => 45.0,
        ShipClass::Cruiser => 70.0,
        ShipClass::Hauler => 60.0,
    };

    // Créer un triangle
    let vertices: Vec<[f32; 3]> = vec![
        [0.0, size / 2.0, 0.0],
        [-size * 0.35, -size / 2.0, 0.0],
        [size * 0.35, -size / 2.0, 0.0],
    ];

    let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(bevy::render::mesh::Mesh::ATTRIBUTE_POSITION, vertices);

    let material = if is_local {
        assets.player_material.clone()
    } else if matches!(entity_type, EntityType::NpcHostile) {
        assets.hostile_material.clone()
    } else {
        assets
            .faction_materials
            .get(&faction)
            .cloned()
            .unwrap_or_else(|| assets.default_material.clone())
    };

    let mesh_id = meshes.add(mesh);

    let mut entity = commands.spawn((
        Mesh2d(mesh_id),
        MeshMaterial2d(material),
        Transform::from_translation(bevy::math::Vec3::new(
            position.x, position.y, 0.0,
        ))
        .with_rotation(Quat::from_rotation_z(rotation.angle)),
        // Composants de jeu
        ShipRender::new(ship_class, faction, entity_type),
        Position::new(position.x, position.y, position.z),
        Rotation::new(rotation.angle),
        Velocity::zero(),
        stats,
        ShipData::fighter(),
        CombatState::new(),
        NetworkId(network_id),
    ));

    if is_local {
        entity.insert(LocalPlayer);
    }

    entity.id()
}
