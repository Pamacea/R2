//! Interpolation réseau pour un mouvement fluide
//!
//! Utilise l'interpolation cubique de Hermite pour lisser les mouvements
//! des entités contrôlées par le serveur.

use bevy::prelude::*;
use r2_core::components::*;
use std::collections::HashMap;

/// Configuration de l'interpolation
#[derive(Resource, Clone, Debug)]
pub struct InterpolationConfig {
    /// Délai d'interpolation en secondes (100ms par défaut)
    /// Cela permet d'avoir toujours des positions futures à interpoler
    pub delay_seconds: f32,
    /// Temps maximum pour extrapoler (si pas de nouveaux paquets)
    pub max_extrapolation: f32,
}

impl Default for InterpolationConfig {
    fn default() -> Self {
        Self {
            delay_seconds: 0.1,  // 100ms
            max_extrapolation: 0.5,  // 500ms max extrapolation
        }
    }
}

/// Snapshot d'une entité à un instant donné
#[derive(Clone, Debug)]
pub struct EntitySnapshot {
    /// Position
    pub position: Position,
    /// Rotation
    pub rotation: Rotation,
    /// Vélocité
    pub velocity: Velocity,
    /// Timestamp (en secondes depuis le démarrage)
    pub timestamp: f32,
    /// Numéro de tick serveur
    pub tick: u64,
}

impl EntitySnapshot {
    /// Crée un nouveau snapshot
    pub fn new(position: Position, rotation: Rotation, velocity: Velocity, timestamp: f32, tick: u64) -> Self {
        Self {
            position,
            rotation,
            velocity,
            timestamp,
            tick,
        }
    }

    /// Interpole entre deux snapshots (Hermite)
    ///
    /// # Arguments
    /// * `other` - L'autre snapshot
    /// * `t` - Facteur d'interpolation (0.0 = self, 1.0 = other)
    pub fn interpolate(&self, other: &EntitySnapshot, t: f32) -> EntitySnapshot {
        // Clamp t entre 0 et 1
        let t = t.clamp(0.0, 1.0);

        // Interpolation cubique de Hermite pour plus de fluidité
        // smoothstep(t) = 3t² - 2t³
        let smooth_t = t * t * (3.0 - 2.0 * t);

        EntitySnapshot {
            position: Position {
                x: self.position.x + (other.position.x - self.position.x) * smooth_t,
                y: self.position.y + (other.position.y - self.position.y) * smooth_t,
                z: self.position.z + (other.position.z - self.position.z) * smooth_t,
            },
            rotation: Rotation {
                angle: shortest_angle_interpolation(
                    self.rotation.angle,
                    other.rotation.angle,
                    smooth_t,
                ),
            },
            velocity: Velocity {
                x: self.velocity.x + (other.velocity.x - self.velocity.x) * smooth_t,
                y: self.velocity.y + (other.velocity.y - self.velocity.y) * smooth_t,
            },
            timestamp: self.timestamp + (other.timestamp - self.timestamp) * t,
            tick: self.tick, // Le tick n'est pas interpolé
        }
    }

    /// Extrapolate dans le temps en utilisant la vélocité
    pub fn extrapolate(&self, delta: f32) -> EntitySnapshot {
        EntitySnapshot {
            position: Position {
                x: self.position.x + self.velocity.x * delta,
                y: self.position.y + self.velocity.y * delta,
                z: self.position.z,
            },
            rotation: self.rotation,
            velocity: self.velocity,
            timestamp: self.timestamp + delta,
            tick: self.tick,
        }
    }
}

/// Interpolation d'angle la plus courte
fn shortest_angle_interpolation(a0: f32, a1: f32, t: f32) -> f32 {
    let max_angle = std::f32::consts::PI * 2.0;
    let da = (a1 - a0) % max_angle;
    let distance = 2.0 * da % max_angle - da;
    a0 + distance * t
}

/// Buffer de snapshots pour une entité
#[derive(Clone, Debug)]
pub struct SnapshotBuffer {
    /// Snapshots stockés (ordonnés par timestamp)
    snapshots: Vec<EntitySnapshot>,
    /// Capacité maximale du buffer
    capacity: usize,
}

impl SnapshotBuffer {
    /// Crée un nouveau buffer
    pub fn new(capacity: usize) -> Self {
        Self {
            snapshots: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Ajoute un snapshot au buffer
    pub fn push(&mut self, snapshot: EntitySnapshot) {
        // Trouver la position d'insertion pour maintenir l'ordre
        let pos = self
            .snapshots
            .partition_point(|s| s.timestamp < snapshot.timestamp);

        // Insérer à la bonne position
        self.snapshots.insert(pos, snapshot);

        // Nettoyer les anciens snapshots (garder les 2 plus récents)
        while self.snapshots.len() > self.capacity {
            self.snapshots.remove(0);
        }
    }

    /// Trouve les deux snapshots autour d'un temps donné
    ///
    /// Retourne (snapshot_before, snapshot_after, factor_t)
    /// où factor_t est entre 0 et 1
    pub fn find_for_time(&self, target_time: f32) -> Option<(EntitySnapshot, EntitySnapshot, f32)> {
        if self.snapshots.len() < 2 {
            return None;
        }

        // Trouver le snapshot juste avant target_time
        let after_idx = self
            .snapshots
            .partition_point(|s| s.timestamp < target_time);

        if after_idx == 0 {
            // Tous les snapshots sont dans le futur
            // Extrapoler depuis le plus ancien
            let snapshot = self.snapshots.first()?;
            let delta = target_time - snapshot.timestamp;
            return Some((
                snapshot.clone(),
                snapshot.clone(),
                delta,
            ));
        }

        if after_idx >= self.snapshots.len() {
            // Tous les snapshots sont dans le passé
            // Extrapoler depuis le plus récent
            let snapshot = self.snapshots.last()?;
            let delta = target_time - snapshot.timestamp;
            return Some((
                snapshot.clone(),
                snapshot.clone(),
                delta,
            ));
        }

        // Interpolation normale entre deux snapshots
        let before = &self.snapshots[after_idx - 1];
        let after = &self.snapshots[after_idx];

        let time_range = after.timestamp - before.timestamp;
        let t = if time_range > 0.0 {
            (target_time - before.timestamp) / time_range
        } else {
            0.0
        };

        Some((before.clone(), after.clone(), t))
    }

    /// Retourne le timestamp du snapshot le plus récent
    pub fn latest_timestamp(&self) -> Option<f32> {
        self.snapshots.last().map(|s| s.timestamp)
    }

    /// Vide le buffer
    pub fn clear(&mut self) {
        self.snapshots.clear();
    }
}

/// Ressource contenant les buffers d'interpolation pour toutes les entités
#[derive(Resource, Default, Debug)]
pub struct InterpolationState {
    /// Map: entity_id -> buffer de snapshots
    pub buffers: HashMap<u64, SnapshotBuffer>,
}

impl InterpolationState {
    /// Ajoute un snapshot pour une entité
    pub fn add_snapshot(&mut self, entity_id: u64, snapshot: EntitySnapshot) {
        let buffer = self.buffers.entry(entity_id).or_insert_with(|| SnapshotBuffer::new(10));
        buffer.push(snapshot);
    }

    /// Récupère la position interpolée pour une entité à un temps donné
    pub fn get_interpolated(
        &self,
        entity_id: u64,
        target_time: f32,
    ) -> Option<EntitySnapshot> {
        let buffer = self.buffers.get(&entity_id)?;
        let (before, after, t) = buffer.find_for_time(target_time)?;

        if t > 1.0 {
            // Extrapolation
            Some(before.extrapolate(t - 1.0))
        } else if t < 0.0 {
            // Extrapolation arrière
            Some(before.extrapolate(t))
        } else {
            // Interpolation normale
            Some(before.interpolate(&after, t))
        }
    }

    /// Supprime une entité du système d'interpolation
    pub fn remove_entity(&mut self, entity_id: u64) {
        self.buffers.remove(&entity_id);
    }

    /// Nettoie les anciennes snapshots
    pub fn cleanup(&mut self, before_time: f32) {
        for buffer in self.buffers.values_mut() {
            buffer.snapshots.retain(|s| s.timestamp >= before_time);
        }
    }
}

/// Plugin d'interpolation
pub struct InterpolationPlugin;

impl Plugin for InterpolationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InterpolationConfig>()
            .init_resource::<InterpolationState>()
            .add_systems(PostUpdate, cleanup_old_snapshots);
    }
}

/// Nettoyage périodique des anciens snapshots
fn cleanup_old_snapshots(
    _config: Res<InterpolationConfig>,
    time: Res<Time>,
    mut state: ResMut<InterpolationState>,
) {
    // Nettoyer toutes les secondes
    let elapsed = time.elapsed_secs() as f32;
    if elapsed % 1.0 < 0.1 {
        let cleanup_time = elapsed - 10.0; // Garder 10 secondes d'historique
        state.cleanup(cleanup_time);
    }
}

/// Composant pour marquer une entité comme interpolée par le réseau
#[derive(Component, Clone, Debug)]
pub struct NetworkInterpolated {
    /// ID réseau de l'entité
    pub network_id: u64,
}

/// Système d'interpolation pour les entités réseau
pub fn interpolate_entities(
    config: Res<InterpolationConfig>,
    time: Res<Time>,
    state: Res<InterpolationState>,
    mut query: Query<(&mut Position, &mut Rotation, &mut Velocity, &NetworkInterpolated)>,
) {
    let render_time = time.elapsed_secs() as f32 - config.delay_seconds;

    for (mut pos, mut rot, mut vel, net) in query.iter_mut() {
        if let Some(snapshot) = state.get_interpolated(net.network_id, render_time) {
            *pos = snapshot.position;
            *rot = snapshot.rotation;
            *vel = snapshot.velocity;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_interpolation() {
        let s1 = EntitySnapshot::new(
            Position::new(0.0, 0.0, 0.0),
            Rotation::new(0.0),
            Velocity::new(1.0, 0.0),
            0.0,
            0,
        );

        let s2 = EntitySnapshot::new(
            Position::new(10.0, 0.0, 0.0),
            Rotation::new(std::f32::consts::PI),
            Velocity::new(1.0, 0.0),
            1.0,
            1,
        );

        let result = s1.interpolate(&s2, 0.5);
        assert!((result.position.x - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_buffer_insertion() {
        let mut buffer = SnapshotBuffer::new(10);

        buffer.push(EntitySnapshot::new(
            Position::new(0.0, 0.0, 0.0),
            Rotation::new(0.0),
            Velocity::zero(),
            1.0,
            0,
        ));

        buffer.push(EntitySnapshot::new(
            Position::new(10.0, 0.0, 0.0),
            Rotation::new(0.0),
            Velocity::zero(),
            0.5,
            1,
        ));

        // Vérifier que les snapshots sont ordonnés par timestamp
        assert!(buffer.snapshots[0].timestamp < buffer.snapshots[1].timestamp);
    }
}
