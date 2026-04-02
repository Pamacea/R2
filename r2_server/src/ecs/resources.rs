//! Ressources globales du ECS
//!
//! Ressources partagées entre tous les systèmes:
//! - DeltaTime pour les calculs de temps
//! - EntityMap pour le mapping Entity Bevy -> EntityId réseau
//! - NetworkUpdates pour collecter les updates à envoyer

use bevy_ecs::{entity::Entity, system::Resource};
use r2_core::{Position, Rotation, Velocity};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ============================================================================
// TIME RESOURCES
// ============================================================================

/// Temps écoulé depuis le dernier tick (fixe à 1/60s)
#[derive(Clone, Copy, Debug, Resource)]
pub struct DeltaTime(pub f32);

impl Default for DeltaTime {
    fn default() -> Self {
        Self(1.0 / 60.0)
    }
}

/// Temps de jeu global
#[derive(Clone, Debug, Resource)]
pub struct GameTime {
    /// Instant du dernier tick
    pub last_tick: std::time::Instant,
    /// Temps total écoulé (secondes)
    pub total_time: f32,
    /// Numéro de tick actuel
    pub tick_number: u64,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            last_tick: std::time::Instant::now(),
            total_time: 0.0,
            tick_number: 0,
        }
    }
}

impl GameTime {
    /// Avancer le temps d'un tick
    pub fn tick(&mut self) {
        self.tick_number += 1;
        let now = std::time::Instant::now();
        self.total_time += now.duration_since(self.last_tick).as_secs_f32();
        self.last_tick = now;
    }
}

// ============================================================================
// ENTITY MAPPING
// ============================================================================

/// Mapping Entity Bevy -> EntityId réseau / PlayerId
///
/// Permet de faire le lien entre les entités ECS et les identifiants réseau.
#[derive(Clone, Default, Debug, Resource)]
pub struct EntityMap(pub HashMap<Entity, u64>);

impl EntityMap {
    /// Associer une Entity Bevy à un ID réseau
    pub fn insert(&mut self, entity: Entity, id: u64) {
        self.0.insert(entity, id);
    }

    /// Obtenir l'ID réseau d'une Entity
    pub fn get(&self, entity: Entity) -> Option<u64> {
        self.0.get(&entity).copied()
    }

    /// Retirer une Entity du mapping
    pub fn remove(&mut self, entity: Entity) -> Option<u64> {
        self.0.remove(&entity)
    }

    /// Créer un mapping inverse (ID -> Entity)
    pub fn inverse(&self) -> HashMap<u64, Entity> {
        self.0.iter().map(|(k, v)| (*v, *k)).collect()
    }
}

/// Prochain ID d'entité réseau disponible
#[derive(Clone, Default, Debug, Resource)]
pub struct NextEntityId(pub u64);

impl NextEntityId {
    /// Obtenir le prochain ID et incrémenter
    pub fn next(&mut self) -> u64 {
        let id = self.0;
        self.0 += 1;
        id
    }
}

// ============================================================================
// NETWORK UPDATES
// ============================================================================

/// Update d'entité à envoyer aux clients
#[derive(Clone, Debug)]
pub struct NetworkEntityUpdate {
    /// ID réseau de l'entité
    pub id: u64,
    /// Nouvelle position (optionnelle)
    pub position: Option<(f32, f32, f32)>,
    /// Nouvelle rotation (optionnelle)
    pub rotation: Option<f32>,
    /// Nouvelle vélocité (optionnelle)
    pub velocity: Option<(f32, f32)>,
    /// Delta de stats (optionnel)
    pub stats_delta: Option<StatsDelta>,
}

/// Delta de stats pour optimiser le réseau
#[derive(Clone, Copy, Debug, Default)]
pub struct StatsDelta {
    pub hull_delta: f32,
    pub shield_delta: f32,
    pub energy_delta: f32,
}

impl StatsDelta {
    pub fn new(hull: f32, shield: f32, energy: f32) -> Self {
        Self {
            hull_delta: hull,
            shield_delta: shield,
            energy_delta: energy,
        }
    }

    /// Vérifier si le delta est significatif
    pub fn is_significant(&self) -> bool {
        self.hull_delta.abs() > 0.01
            || self.shield_delta.abs() > 0.01
            || self.energy_delta.abs() > 0.01
    }
}

/// Buffer des updates réseau à envoyer aux clients
#[derive(Clone, Default, Debug, Resource)]
pub struct NetworkUpdates {
    pub updates: Vec<NetworkEntityUpdate>,
}

impl NetworkUpdates {
    /// Ajouter un update de position
    pub fn push_position(&mut self, id: u64, pos: Position) {
        self.updates.push(NetworkEntityUpdate {
            id,
            position: Some((pos.x, pos.y, pos.z)),
            rotation: None,
            velocity: None,
            stats_delta: None,
        });
    }

    /// Ajouter un update complet
    pub fn push(&mut self, update: NetworkEntityUpdate) {
        self.updates.push(update);
    }

    /// Vider le buffer et retourner les updates
    pub fn drain(&mut self) -> Vec<NetworkEntityUpdate> {
        std::mem::take(&mut self.updates)
    }

    /// Nombre d'updates en attente
    pub fn len(&self) -> usize {
        self.updates.len()
    }

    /// Vérifier si le buffer est vide
    pub fn is_empty(&self) -> bool {
        self.updates.is_empty()
    }
}

// ============================================================================
// SPAWN / DESPAWN QUEUES
// ============================================================================

/// Demande de spawn d'entité
#[derive(Clone, Debug)]
pub struct SpawnRequest {
    /// Position initiale
    pub position: Position,
    /// Rotation initiale
    pub rotation: Rotation,
    /// Vélocité initiale
    pub velocity: Velocity,
    /// Player ID (si c'est un joueur)
    pub player_id: Option<i64>,
}

impl SpawnRequest {
    /// Spawn d'un joueur
    pub fn player(player_id: i64, position: Position) -> Self {
        Self {
            position,
            rotation: Rotation::new(0.0),
            velocity: Velocity::zero(),
            player_id: Some(player_id),
        }
    }

    /// Spawn d'un NPC
    pub fn npc(position: Position) -> Self {
        Self {
            position,
            rotation: Rotation::new(0.0),
            velocity: Velocity::zero(),
            player_id: None,
        }
    }
}

/// File d'attente des spawns
#[derive(Clone, Default, Debug)]
pub struct SpawnQueue {
    pub spawns: Vec<SpawnRequest>,
}

impl SpawnQueue {
    pub fn push(&mut self, request: SpawnRequest) {
        self.spawns.push(request);
    }

    pub fn drain(&mut self) -> Vec<SpawnRequest> {
        std::mem::take(&mut self.spawns)
    }
}

/// File d'attente des despawns
#[derive(Clone, Default, Debug)]
pub struct DespawnQueue {
    pub entities: Vec<Entity>,
}

impl DespawnQueue {
    pub fn push(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn drain(&mut self) -> Vec<Entity> {
        std::mem::take(&mut self.entities)
    }
}

// ============================================================================
// WORLD BOUNDARIES
// ============================================================================

/// Limites du monde de jeu
#[derive(Clone, Copy, Debug, Resource)]
pub struct WorldBounds {
    /// Coordonnées X min/max
    pub x_min: f32,
    pub x_max: f32,
    /// Coordonnées Y min/max
    pub y_min: f32,
    pub y_max: f32,
    /// Rayon de suppression (hors limites)
    pub despawn_radius: f32,
}

impl Default for WorldBounds {
    fn default() -> Self {
        Self {
            x_min: -50000.0,
            x_max: 50000.0,
            y_min: -50000.0,
            y_max: 50000.0,
            despawn_radius: 55000.0,
        }
    }
}

impl WorldBounds {
    /// Vérifier si une position est dans les limites
    pub fn is_inside(&self, x: f32, y: f32) -> bool {
        x >= self.x_min && x <= self.x_max && y >= self.y_min && y <= self.y_max
    }

    /// Vérifier si une position est hors limites (à désparer)
    pub fn is_outside(&self, x: f32, y: f32) -> bool {
        x < self.x_min || x > self.x_max || y < self.y_min || y > self.y_max
    }

    /// Vérifier si une position est dans la zone de désapparition
    pub fn is_despawn_zone(&self, x: f32, y: f32) -> bool {
        let dist = (x * x + y * y).sqrt();
        dist > self.despawn_radius
    }

    /// Clamp une position dans les limites
    pub fn clamp(&self, x: f32, y: f32) -> (f32, f32) {
        (
            x.clamp(self.x_min, self.x_max),
            y.clamp(self.y_min, self.y_max),
        )
    }
}

// ============================================================================
// PLAYER CONNECTIONS
// ============================================================================

/// État des connexions joueurs (Thread-safe pour accès async)
///
/// Ressource partagée entre le ECS (simulation) et le handler WebSocket (async).
#[derive(Clone, Debug, Resource)]
pub struct PlayerConnections {
    /// Mapping PlayerId -> Entity Bevy
    pub players: Arc<Mutex<HashMap<i64, PlayerConnectionData>>>,
}

/// Données de connexion d'un joueur
#[derive(Clone, Debug)]
pub struct PlayerConnectionData {
    /// Entity Bevy du joueur
    pub entity: Entity,
    /// ID réseau de l'entité
    pub network_id: u64,
    /// Timestamp de dernière activité
    pub last_update: std::time::Instant,
    /// État de connexion
    pub connected: bool,
}

impl PlayerConnectionData {
    pub fn new(entity: Entity, network_id: u64) -> Self {
        Self {
            entity,
            network_id,
            last_update: std::time::Instant::now(),
            connected: true,
        }
    }

    /// Marquer une activité (ping/pong)
    pub fn mark_activity(&mut self) {
        self.last_update = std::time::Instant::now();
    }

    /// Vérifier si le joueur est timeout (> 30s sans activité)
    pub fn is_timeout(&self) -> bool {
        self.last_update.elapsed() > std::time::Duration::from_secs(30)
    }
}

impl Default for PlayerConnections {
    fn default() -> Self {
        Self {
            players: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl PlayerConnections {
    /// Connecter un joueur
    pub fn connect(&self, player_id: i64, entity: Entity, network_id: u64) {
        let mut players = self.players.lock().unwrap();
        players.insert(player_id, PlayerConnectionData::new(entity, network_id));
    }

    /// Déconnecter un joueur
    pub fn disconnect(&self, player_id: i64) {
        let mut players = self.players.lock().unwrap();
        if let Some(data) = players.get_mut(&player_id) {
            data.connected = false;
        }
    }

    /// Obtenir l'Entity d'un joueur
    pub fn get_entity(&self, player_id: i64) -> Option<Entity> {
        let players = self.players.lock().unwrap();
        players.get(&player_id).map(|data| data.entity)
    }

    /// Obtenir l'ID réseau d'un joueur
    pub fn get_network_id(&self, player_id: i64) -> Option<u64> {
        let players = self.players.lock().unwrap();
        players.get(&player_id).map(|data| data.network_id)
    }

    /// Vérifier si un joueur est connecté
    pub fn is_connected(&self, player_id: i64) -> bool {
        let players = self.players.lock().unwrap();
        players.get(&player_id).map(|d| d.connected).unwrap_or(false)
    }

    /// Nettoyer les connexions timeout
    pub fn cleanup_timeouts(&self) -> Vec<i64> {
        let mut players = self.players.lock().unwrap();
        let mut timeout_players = Vec::new();

        players.retain(|player_id, data| {
            if data.is_timeout() {
                timeout_players.push(*player_id);
                false
            } else {
                true
            }
        });

        timeout_players
    }

    /// Nombre de joueurs connectés
    pub fn connected_count(&self) -> usize {
        let players = self.players.lock().unwrap();
        players.values().filter(|d| d.connected).count()
    }
}

// ============================================================================
// AREA OF INTEREST (AOI)
// ============================================================================

/// Configuration pour le Area of Interest
#[derive(Clone, Copy, Debug, Resource)]
pub struct AoiConfig {
    /// Rayon de vision d'un joueur (unités monde)
    pub view_radius: f32,
    /// Marge pour éviter le clignotement aux bordures
    pub margin: f32,
}

impl Default for AoiConfig {
    fn default() -> Self {
        Self {
            view_radius: 2000.0,
            margin: 200.0,
        }
    }
}

impl AoiConfig {
    /// Rayon effectif avec marge
    pub fn effective_radius(&self) -> f32 {
        self.view_radius + self.margin
    }
}

// ============================================================================
// COMBAT CONFIG
// ============================================================================

/// Configuration des paramètres de combat
#[derive(Clone, Copy, Debug, Resource)]
pub struct CombatConfig {
    /// Dégâts de collision (par unité de vitesse)
    pub collision_damage_multiplier: f32,
    /// Timer avant sortie de combat (secondes)
    pub combat_timeout: f32,
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            collision_damage_multiplier: 10.0,
            combat_timeout: 15.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_map() {
        let mut map = EntityMap::default();
        let entity = Entity::from_raw(1);

        map.insert(entity, 123);
        assert_eq!(map.get(entity), Some(123));

        let inverse = map.inverse();
        assert_eq!(inverse.get(&123), Some(&entity));
    }

    #[test]
    fn test_world_bounds() {
        let bounds = WorldBounds::default();

        assert!(bounds.is_inside(0.0, 0.0));
        assert!(bounds.is_inside(1000.0, -1000.0));
        assert!(!bounds.is_inside(60000.0, 0.0));

        let (clamped_x, clamped_y) = bounds.clamp(60000.0, -60000.0);
        assert_eq!(clamped_x, 50000.0);
        assert_eq!(clamped_y, -50000.0);
    }

    #[test]
    fn test_network_updates() {
        let mut updates = NetworkUpdates::default();

        updates.push_position(1, Position::new(10.0, 20.0, 30.0));
        assert_eq!(updates.len(), 1);

        let drained = updates.drain();
        assert_eq!(drained.len(), 1);
        assert!(updates.is_empty());
    }

    #[test]
    fn test_player_connections() {
        let connections = PlayerConnections::default();
        let entity = Entity::from_raw(1);

        connections.connect(123, entity, 456);
        assert!(connections.is_connected(123));
        assert_eq!(connections.get_entity(123), Some(entity));
        assert_eq!(connections.get_network_id(123), Some(456));
        assert_eq!(connections.connected_count(), 1);

        connections.disconnect(123);
        assert!(!connections.is_connected(123));
    }
}
