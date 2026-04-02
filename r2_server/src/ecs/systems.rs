//! Systèmes ECS pour la simulation de jeu
//!
//! Systèmes exécutés à 60Hz:
//! - Mouvement physique
//! - Collision détection
//! - Combat (armes, projectiles)
//! - Régénération

use bevy_ecs::entity::Entity;

use r2_core::{CombatState, Position, Rotation, Stats, Velocity};

use super::components::*;
use super::resources::*;

// ============================================================================
// MOVEMENT SYSTEMS
// ============================================================================

/// Système de mouvement basé sur la vélocité
pub fn movement_system(world: &mut World) {
    let delta = {
        let time = world.get_resource::<DeltaTime>().unwrap();
        time.0
    };

    // Collecter les entités avec Position et Velocity
    let mut entities_data = Vec::new();
    {
        let mut query = world.query::<(&Position, &Velocity)>();
        for (pos, vel) in query.iter(world) {
            entities_data.push((pos.clone(), vel.clone()));
        }
    }

    // Appliquer les mouvements
    {
        let mut query = world.query::<(&mut Position, &Velocity)>();
        for (mut pos, vel) in query.iter_mut(world) {
            pos.x += vel.x * delta;
            pos.y += vel.y * delta;
        }
    }
}

/// Système de rotation vers une cible
pub fn rotation_system(_world: &mut World) {
    // TODO: Implémenter la rotation des entités avec Propulsion
}

// ============================================================================
// COLLISION SYSTEMS
// ============================================================================

/// Système de limitation aux bornes du monde
pub fn boundary_system(world: &mut World) {
    let bounds = world.get_resource::<WorldBounds>().unwrap();
    let mut despawn_queue = world.get_resource_mut::<DespawnQueue>().unwrap();

    let mut entities_to_despawn = Vec::new();

    {
        let mut query = world.query::<(Entity, &Position, &Velocity)>();
        for (entity, pos, vel) in query.iter(world) {
            // Vérifier si dans la zone de désaparition
            if bounds.is_despawn_zone(pos.x, pos.y) {
                entities_to_despawn.push(entity);
                continue;
            }

            // Clamp aux limites du monde
            let (clamped_x, clamped_y) = bounds.clamp(pos.x, pos.y);

            // Rebondir sur les bords
            // TODO: Appliquer les changements de vélocité
            let _ = (clamped_x, clamped_y, vel);
        }
    }

    for entity in entities_to_despawn {
        despawn_queue.entities.push(entity);
    }
}

// ============================================================================
// COMBAT SYSTEMS
// ============================================================================

/// Système de cooldown des armes
pub fn weapon_cooldown_system(world: &mut World) {
    let delta = {
        let time = world.get_resource::<DeltaTime>().unwrap();
        time.0
    };

    let mut query = world.query::<&Weapon>();
    for mut weapon in query.iter_mut(world) {
        weapon.update(delta);
    }
}

/// Système de mouvement des projectiles
pub fn projectile_movement_system(world: &mut World) {
    let delta = {
        let time = world.get_resource::<DeltaTime>().unwrap();
        time.0
    };

    let mut despawn_queue = world.get_resource_mut::<DespawnQueue>().unwrap();
    let mut entities_to_despawn = Vec::new();

    {
        let mut query = world.query::<(Entity, &Position, &Velocity, &Projectile)>();
        for (entity, pos, vel, proj) in query.iter(world) {
            // Mouvement
            let new_x = pos.x + vel.x * delta;
            let new_y = pos.y + vel.y * delta;

            // Durée de vie
            if proj.lifetime <= 0.0 {
                entities_to_despawn.push(entity);
            }

            // TODO: Appliquer les changements de position
            let _ = (new_x, new_y);
        }
    }

    for entity in entities_to_despawn {
        world.despawn(entity);
        despawn_queue.entities.push(entity);
    }
}

/// Système de collision des projectiles
pub fn projectile_collision_system(world: &mut World) {
    // Collecter les projectiles
    let mut projectiles = Vec::new();
    {
        let mut query = world.query::<(Entity, &Position, &CollisionSphere, &Projectile)>();
        for (entity, pos, col, proj) in query.iter(world) {
            projectiles.push((entity, pos.clone(), col.clone(), proj.clone()));
        }
    }

    // Collecter les cibles
    let mut targets = Vec::new();
    {
        let mut query = world.query::<(Entity, &Position, &CollisionSphere)>();
        for (entity, pos, col) in query.iter(world) {
            targets.push((entity, pos.clone(), col.clone()));
        }
    }

    let mut entity_map = world.get_resource_mut::<EntityMap>().unwrap();
    let mut updates = world.get_resource_mut::<NetworkUpdates>().unwrap();

    // Vérifier les collisions
    for (proj_entity, proj_pos, proj_col, proj_data) in &projectiles {
        for (target_entity, target_pos, target_col) in &targets {
            if proj_entity == target_entity {
                continue;
            }

            if proj_col.collides_with(target_pos, proj_pos, target_col) {
                // Appliquer les dégâts
                if let Ok(mut stats) = world.get_mut::<Stats>(*target_entity) {
                    let damage = stats.take_damage(proj_data.damage);

                    // Marquer pour update réseau
                    if let Some(network_id) = entity_map.get(*target_entity) {
                        updates.push(NetworkEntityUpdate {
                            id: network_id,
                            position: None,
                            rotation: None,
                            velocity: None,
                            stats_delta: Some(StatsDelta {
                                hull_delta: -damage,
                                shield_delta: 0.0,
                                energy_delta: 0.0,
                            }),
                        });
                    }
                }

                // Détruire le projectile
                world.despawn(*proj_entity);
                break;
            }
        }
    }
}

// ============================================================================
// REGEN SYSTEMS
// ============================================================================

/// Système de régénération des stats
pub fn stats_regen_system(world: &mut World) {
    let delta = {
        let time = world.get_resource::<DeltaTime>().unwrap();
        time.0
    };

    let entity_map = world.get_resource::<EntityMap>().unwrap();
    let mut updates = world.get_resource_mut::<NetworkUpdates>().unwrap();

    let mut query = world.query::<(Entity, &Stats)>();
    for (entity, stats) in query.iter(world) {
        let old_hull = stats.hull_current;
        let old_shield = stats.shield_current;
        let old_energy = stats.energy_current;

        // TODO: Appliquer la regen
        // stats.regen(delta);

        let hull_delta = stats.hull_current - old_hull;
        let shield_delta = stats.shield_current - old_shield;
        let energy_delta = stats.energy_current - old_energy;

        if hull_delta.abs() > 0.01 || shield_delta.abs() > 0.01 || energy_delta.abs() > 0.01 {
            if let Some(network_id) = entity_map.get(entity) {
                updates.push(NetworkEntityUpdate {
                    id: network_id,
                    position: None,
                    rotation: None,
                    velocity: None,
                    stats_delta: Some(StatsDelta::new(hull_delta, shield_delta, energy_delta)),
                });
            }
        }
    }
}

/// Système de timer de combat
pub fn combat_timer_system(world: &mut World) {
    let delta = {
        let time = world.get_resource::<DeltaTime>().unwrap();
        time.0
    };

    let mut query = world.query::<&CombatState>();
    for mut combat in query.iter_mut(world) {
        combat.update(delta);
    }
}

// ============================================================================
// DESPAWN SYSTEMS
// ============================================================================

/// Système de suppression des entités marquées
pub fn despawn_system(world: &mut World) {
    let despawn_queue = world.get_resource::<DespawnQueue>().unwrap();
    let entities = despawn_queue.entities.clone();
    drop(despawn_queue);

    let mut entity_map = world.get_resource_mut::<EntityMap>().unwrap();

    for entity in entities {
        entity_map.0.remove(&entity);
        let _ = world.despawn(entity);
    }

    // Vider la queue
    let mut despawn_queue = world.get_resource_mut::<DespawnQueue>().unwrap();
    despawn_queue.entities.clear();
}

// ============================================================================
// NETWORK SYNC SYSTEMS
// ============================================================================

/// Système de synchronisation réseau
pub fn network_sync_system(world: &mut World) {
    let entity_map = world.get_resource::<EntityMap>().unwrap();
    let mut updates = world.get_resource_mut::<NetworkUpdates>().unwrap();

    let mut query = world.query::<(Entity, &Position, &Rotation, &Velocity, &NetworkSync)>();
    for (entity, pos, rot, vel, sync) in query.iter(world) {
        if !sync.dirty {
            continue;
        }

        if let Some(network_id) = entity_map.get(entity) {
            updates.push(NetworkEntityUpdate {
                id: network_id,
                position: Some((pos.x, pos.y, pos.z)),
                rotation: Some(rot.angle),
                velocity: Some((vel.x, vel.y)),
                stats_delta: None,
            });
        }
    }
}

// ============================================================================
// SCHEDULE
// ============================================================================

/// Scheduler principal - exécute tous les systèmes dans l'ordre
pub fn run_schedule(world: &mut World) {
    // 1. Movement
    movement_system(world);
    rotation_system(world);

    // 2. Combat
    weapon_cooldown_system(world);
    projectile_movement_system(world);
    projectile_collision_system(world);

    // 3. Regeneration
    stats_regen_system(world);
    combat_timer_system(world);

    // 4. World bounds
    boundary_system(world);

    // 5. Cleanup
    despawn_system(world);

    // 6. Network sync
    network_sync_system(world);
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use r2_core::Position;

    #[test]
    fn test_weapon_cooldown() {
        let mut weapon = Weapon::laser(0);
        assert!(weapon.is_ready());

        weapon.fire();
        assert!(!weapon.is_ready());
        assert_eq!(weapon.cooldown_remaining, 0.5);

        weapon.update(0.3);
        assert!(!weapon.is_ready());

        weapon.update(0.3);
        assert!(weapon.is_ready());
    }

    #[test]
    fn test_stats_regen() {
        let mut stats = Stats::new(100.0, 50.0, 100.0);

        stats.shield_current = 10.0;
        stats.energy_current = 20.0;

        stats.regen(1.0); // 1 seconde

        // Shield regen 5/s, energy regen 10/s
        assert_eq!(stats.shield_current, 15.0);
        assert_eq!(stats.energy_current, 30.0);
    }

    #[test]
    fn test_collision_sphere() {
        let sphere1 = CollisionSphere::new(10.0, CollisionType::Ship);
        let sphere2 = CollisionSphere::new(15.0, CollisionType::Ship);

        let pos1 = Position::new(0.0, 0.0, 0.0);
        let pos2 = Position::new(20.0, 0.0, 0.0);

        // Distance = 20, rayons = 25 -> collision
        assert!(sphere1.collides_with(&pos2, &pos1, &sphere2));

        let pos3 = Position::new(30.0, 0.0, 0.0);
        // Distance = 30, rayons = 25 -> pas de collision
        assert!(!sphere1.collides_with(&pos3, &pos1, &sphere2));
    }
}
