//! Module ECS pour R2 Server
//!
//! Système Entity-Component-System utilisant Bevy ECS (headless, sans rendering).
//! Tourne à 60Hz fixes pour la simulation physique et gameplay.
//!
//! # Architecture
//!
//! ```text
//! ecs/
//!   ├── mod.rs         - Point d'entrée, exports
//!   ├── components.rs  - Composants serveur-only
//!   ├── resources.rs   - Ressources globales (DeltaTimer, EntityMap, etc.)
//!   └── systems.rs     - Systèmes de simulation
//! ```

pub mod components;
pub mod resources;

pub use components::*;

// Note: Les systèmes sont implémentés directement dans EcsRunner pour simplifier
// l'intégration avec l'API bevy_ecs qui a changé dans la version 0.15

use bevy_ecs::{entity::Entity, world::World};
use r2_core::{CombatState, Position, Stats, Velocity};

/// Créer un nouveau World ECS avec toutes les ressources nécessaires
pub fn create_world() -> World {
    let mut world = World::new();

    // Enregistrer les composants personnalisés
    world.register_component::<PlayerId>();
    world.register_component::<Player>();
    world.register_component::<PlayerShip>();
    world.register_component::<Propulsion>();
    world.register_component::<Weapon>();
    world.register_component::<Projectile>();
    world.register_component::<CollisionSphere>();
    world.register_component::<AiBehavior>();
    world.register_component::<NetworkSync>();
    world.register_component::<DespawnMarker>();
    world.register_component::<Spawner>();

    // Insérer les ressources par défaut
    world.insert_resource(resources::DeltaTime::default());
    world.insert_resource(resources::GameTime::default());
    world.insert_resource(resources::EntityMap::default());
    world.insert_resource(resources::NextEntityId::default());
    world.insert_resource(resources::NetworkUpdates::default());
    world.insert_resource(resources::WorldBounds::default());

    world
}

/// Runner principal du ECS - exécute la simulation à 60Hz
pub struct EcsRunner {
    pub world: World,
    pub tick_duration: std::time::Duration,
}

impl EcsRunner {
    /// Créer un nouveau runner
    pub fn new() -> Self {
        Self {
            world: create_world(),
            tick_duration: std::time::Duration::from_secs_f64(1.0 / 60.0),
        }
    }

    /// Exécuter un tick de la simulation
    pub fn tick(&mut self) {
        // Mettre à jour le temps
        let now = std::time::Instant::now();
        {
            let mut game_time = self.world.get_resource_mut::<resources::GameTime>().unwrap();
            let delta = now.duration_since(game_time.last_tick);
            game_time.last_tick = now;
            game_time.total_time += delta.as_secs_f32();
            game_time.tick_number += 1;
        }

        // Mettre à jour DeltaTime (fixe à 1/60s)
        {
            let mut delta_time = self.world.get_resource_mut::<resources::DeltaTime>().unwrap();
            delta_time.0 = 1.0 / 60.0;
        }

        // Exécuter les systèmes
        self.run_movement_system();
        self.run_boundary_system();
        self.run_weapon_cooldown_system();
        self.run_projectile_system();
        self.run_regen_system();
        self.run_combat_timer_system();
        self.run_despawn_system();
    }

    fn run_movement_system(&mut self) {
        let delta = 1.0 / 60.0;

        // Collecter les entités avec position et vélocité
        let entities: Vec<_> = {
            let mut query = self.world.query::<(Entity, &Position, &Velocity)>();
            query.iter(&self.world).map(|(e, p, v)| (e, p.to_owned(), v.to_owned())).collect()
        };

        // Appliquer le mouvement
        for (entity, _pos, vel) in entities {
            if let Some(mut pos) = self.world.get_mut::<Position>(entity) {
                pos.x += vel.x * delta;
                pos.y += vel.y * delta;
            }
        }
    }

    fn run_boundary_system(&mut self) {
        let bounds = self.world.get_resource::<resources::WorldBounds>().unwrap();
        let bounds_clone = *bounds;

        let entities: Vec<_> = {
            let mut query = self.world.query::<(Entity, &Position)>();
            query.iter(&self.world)
                .filter(|(_, p)| bounds_clone.is_despawn_zone(p.x, p.y))
                .map(|(e, _)| e)
                .collect()
        };

        for entity in entities {
            let _ = self.world.despawn(entity);
        }
    }

    fn run_weapon_cooldown_system(&mut self) {
        let delta = 1.0 / 60.0;

        let entities: Vec<_> = {
            let mut query = self.world.query::<(Entity, &Weapon)>();
            query.iter(&self.world).map(|(e, _)| e).collect()
        };

        for entity in entities {
            if let Some(mut weapon) = self.world.get_mut::<Weapon>(entity) {
                weapon.update(delta);
            }
        }
    }

    fn run_projectile_system(&mut self) {
        let delta = 1.0 / 60.0;

        let to_despawn: Vec<_> = {
            let mut query = self.world.query::<(Entity, &Projectile)>();
            query.iter(&self.world)
                .filter(|(_, p)| p.lifetime <= delta)
                .map(|(e, _)| e)
                .collect()
        };

        for entity in to_despawn {
            let _ = self.world.despawn(entity);
        }
    }

    fn run_regen_system(&mut self) {
        let delta = 1.0 / 60.0;

        let entities: Vec<_> = {
            let mut query = self.world.query::<(Entity, &Stats)>();
            query.iter(&self.world).map(|(e, _)| e).collect()
        };

        for entity in entities {
            if let Some(mut stats) = self.world.get_mut::<Stats>(entity) {
                stats.regen(delta);
            }
        }
    }

    fn run_combat_timer_system(&mut self) {
        let delta = 1.0 / 60.0;

        let entities: Vec<_> = {
            let mut query = self.world.query::<(Entity, &CombatState)>();
            query.iter(&self.world).map(|(e, _)| e).collect()
        };

        for entity in entities {
            if let Some(mut combat) = self.world.get_mut::<CombatState>(entity) {
                combat.update(delta);
            }
        }
    }

    fn run_despawn_system(&mut self) {
        let to_despawn: Vec<_> = {
            let mut query = self.world.query::<(Entity, &DespawnMarker)>();
            query.iter(&self.world).map(|(e, _)| e).collect()
        };

        for entity in to_despawn {
            let _ = self.world.despawn(entity);
        }
    }

    /// Obtenir les updates réseau à envoyer aux clients
    pub fn drain_network_updates(&mut self) -> Vec<resources::NetworkEntityUpdate> {
        let mut updates = self.world.get_resource_mut::<resources::NetworkUpdates>().unwrap();
        std::mem::take(&mut updates.updates)
    }

    /// Spawner une entité joueur
    pub fn spawn_player(&mut self, player_id: i64, position: (f32, f32, f32)) -> Entity {
        let mut next_id = self.world.get_resource_mut::<resources::NextEntityId>().unwrap();
        let network_id = next_id.next();
        drop(next_id);

        // Spawn d'abord
        let entity = self.world.spawn((
            r2_core::Position::new(position.0, position.1, position.2),
            r2_core::Velocity::zero(),
            r2_core::Rotation::new(0.0),
            r2_core::Stats::new(100.0, 50.0, 100.0),
            r2_core::CombatState::new(),
            components::PlayerId(player_id),
            components::NetworkSync::default(),
        )).id();

        // Puis mapper
        let mut entity_map = self.world.get_resource_mut::<resources::EntityMap>().unwrap();
        entity_map.0.insert(entity, network_id);
        entity
    }

    /// Despawner une entité
    pub fn despawn_entity(&mut self, entity: Entity) {
        let mut entity_map = self.world.get_resource_mut::<resources::EntityMap>().unwrap();
        entity_map.0.remove(&entity);
        drop(entity_map);

        let _ = self.world.despawn(entity);
    }
}

impl Default for EcsRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let world = create_world();

        // Vérifier que les ressources sont présentes
        assert!(world.get_resource::<resources::DeltaTime>().is_some());
        assert!(world.get_resource::<resources::GameTime>().is_some());
        assert!(world.get_resource::<resources::EntityMap>().is_some());
        assert!(world.get_resource::<resources::NextEntityId>().is_some());
    }

    #[test]
    fn test_runner_creation() {
        let runner = EcsRunner::new();
        assert!(runner.world.get_resource::<resources::DeltaTime>().is_some());
    }

    #[test]
    fn test_spawn_player() {
        let mut runner = EcsRunner::new();
        let entity = runner.spawn_player(123, (0.0, 0.0, 0.0));

        // Vérifier que l'entité existe
        assert!(runner.world.get_entity(entity).is_ok());

        // Vérifier les composants
        assert!(runner.world.get::<r2_core::Position>(entity).is_some());
        assert!(runner.world.get::<r2_core::Stats>(entity).is_some());
        assert!(runner.world.get::<components::PlayerId>(entity).is_some());
    }

    #[test]
    fn test_tick_execution() {
        let mut runner = EcsRunner::new();
        let entity = runner.spawn_player(123, (0.0, 0.0, 0.0));

        // Exécuter quelques ticks
        for _ in 0..10 {
            runner.tick();
        }

        // L'entité existe toujours
        assert!(runner.world.get_entity(entity).is_ok());
    }
}
