//! Module d'intégration Game Loop
//!
//! Connecte le ECS, le réseau et la base de données en un game loop cohérent.

use crate::ecs::{EcsRunner, resources::*};
use crate::network::{ServerMessage, EntityData, EntityType};
use crate::db::CharacterRepository;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// État du jeu partagé entre le game loop et les handlers WebSocket
#[derive(Clone)]
pub struct GameState {
    pub ecs: Arc<RwLock<EcsRunner>>,
    pub db: sqlx::SqlitePool,
}

impl GameState {
    pub fn new(db: sqlx::SqlitePool) -> Self {
        Self {
            ecs: Arc::new(RwLock::new(EcsRunner::new())),
            db,
        }
    }

    /// Connecter un joueur et créer son entité dans le monde
    pub async fn spawn_player(&self, player_id: i64, character_id: i64) -> Result<u64, String> {
        // Récupérer la position du personnage depuis la DB
        let char_repo = CharacterRepository::new(self.db.clone());
        let character = char_repo.find_by_id(character_id).await
            .map_err(|e| format!("DB error: {}", e))?
            .ok_or_else(|| format!("Character {} not found", character_id))?;

        let position = (character.x, character.y, character.rotation);

        // Spawn dans le ECS
        let mut ecs = self.ecs.write().await;
        let entity = ecs.spawn_player(player_id, position);

        // Récupérer le network_id
        let entity_map = ecs.world.get_resource::<EntityMap>().unwrap();
        let network_id = entity_map.get(entity).unwrap_or(0);

        info!("Spawned player {} (character {}) at ({}, {}, {}), network_id={}",
              player_id, character_id, position.0, position.1, position.2, network_id);

        Ok(network_id)
    }

    /// Despawner un joueur du monde
    pub async fn despawn_player(&self, player_id: i64, network_id: u64) {
        let mut ecs = self.ecs.write().await;

        // Trouver l'entity Bevy correspondant au network_id
        let entity_to_remove = {
            let entity_map = ecs.world.get_resource::<EntityMap>().unwrap();
            let inverse = entity_map.inverse();
            inverse.get(&network_id).copied()
        };

        if let Some(entity) = entity_to_remove {
            ecs.despawn_entity(entity);
            info!("Despawned player {} (network_id={})", player_id, network_id);
        }
    }

    /// Appliquer un input client à l'entité du joueur
    pub async fn apply_input(&self, player_id: i64, thrust: bool, rotate: f32, fire: bool) {
        let mut ecs = self.ecs.write().await;

        // Trouver l'entité du joueur
        let entity = {
            let player_connections = ecs.world.get_resource::<PlayerConnections>().unwrap();
            player_connections.get_entity(player_id)
        };

        if let Some(entity) = entity {
            // Mettre à jour la vélocité selon l'input
            if let Some(mut vel) = ecs.world.get_mut::<r2_core::Velocity>(entity) {
                // Rotation: angle en radians
                let angle = rotate;

                // Thrust: appliquer une vélocité dans la direction du vaisseau
                let speed = if thrust { 50.0 } else { 0.0 };
                vel.x = angle.cos() * speed;
                vel.y = angle.sin() * speed;
            }

            // Marquer l'entité pour sync réseau
            if let Some(mut sync) = ecs.world.get_mut::<crate::ecs::components::NetworkSync>(entity) {
                sync.mark_dirty();
            }

            // Tir: TODO - implémenter le système d'armes
            if fire {
                debug!("Player {} fired!", player_id);
                // TODO: Créer un projectile
            }
        }
    }

    /// Exécuter un tick du ECS (à 60Hz)
    pub async fn tick(&self) -> Vec<ServerMessage> {
        let mut ecs = self.ecs.write().await;
        ecs.tick();

        // Récupérer les updates réseau
        let updates = ecs.drain_network_updates();

        // Construire les messages WorldUpdate
        // Pour l'instant, un seul message pour tous les updates
        if updates.is_empty() {
            Vec::new()
        } else {
            let entities = updates.into_iter().map(|u| EntityData {
                id: u.id,
                entity_type: EntityType::Player, // TODO: détecter le vrai type
                position: u.position.unwrap_or((0.0, 0.0, 0.0)),
                rotation: u.rotation.unwrap_or(0.0),
                velocity: u.velocity,
                hull_percent: 100.0, // TODO: depuis Stats
                shield_percent: 100.0,
            }).collect();

            vec![ServerMessage::WorldUpdate {
                entities,
                timestamp: chrono::Utc::now().timestamp_millis(),
            }]
        }
    }

    /// Obtenir les entités proches d'une position (AOI)
    pub async fn get_nearby_entities(&self, x: f32, y: f32, radius: f32) -> Vec<EntityData> {
        let mut ecs = self.ecs.write().await;

        // TODO: Implémenter un vrai système de spatial partitioning
        // Pour l'instant, on retourne toutes les entités avec Position
        let mut entities = Vec::new();

        let mut query = ecs.world.query::<(bevy_ecs::entity::Entity, &r2_core::Position, &r2_core::Rotation)>();
        let entity_map = ecs.world.get_resource::<EntityMap>().unwrap();

        for (entity, pos, rot) in query.iter(&ecs.world) {
            // Vérifier la distance
            let dx = pos.x - x;
            let dy = pos.y - y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= radius {
                if let Some(network_id) = entity_map.get(entity) {
                    entities.push(EntityData {
                        id: network_id,
                        entity_type: EntityType::Player,
                        position: (pos.x, pos.y, pos.z),
                        rotation: rot.angle,
                        velocity: None,
                        hull_percent: 100.0,
                        shield_percent: 100.0,
                    });
                }
            }
        }

        entities
    }
}

/// Lancer le game loop en arrière-plan
pub async fn run_game_loop(state: GameState) {
    info!("🎮 Starting game loop at 60Hz...");

    let tick_duration = std::time::Duration::from_secs_f64(1.0 / 60.0);

    loop {
        let start = std::time::Instant::now();

        // Exécuter un tick
        let updates = state.tick().await;

        // TODO: Envoyer les updates aux clients via WebSocket
        if !updates.is_empty() {
            debug!("Generated {} world updates", updates.len());
        }

        // Attendre pour maintenir 60Hz
        let elapsed = start.elapsed();
        if elapsed < tick_duration {
            tokio::time::sleep(tick_duration - elapsed).await;
        }
    }
}
