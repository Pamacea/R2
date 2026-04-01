//! Composants ECS partagés entre client et serveur
//!
//! Ces composants sont sérialisables et peuvent être envoyés sur le réseau.

use serde::{Deserialize, Serialize};
use crate::types::*;
use bevy_ecs::component::Component;

// ============================================================================
// CORE COMPONENTS
// ============================================================================

/// Identifiant unique d'entité réseau
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkId(pub u64);

impl NetworkId {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn is_valid(self) -> bool {
        self.0 != 0
    }
}

impl Default for NetworkId {
    fn default() -> Self {
        Self(0)
    }
}

/// Position dans le monde 3D
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2 { x: self.x, y: self.y }
    }
}

impl From<(f32, f32, f32)> for Position {
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Self { x, y, z }
    }
}

/// Rotation autour de l'axe Z (radians)
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Rotation {
    pub angle: f32,
}

impl Rotation {
    pub fn new(angle: f32) -> Self {
        Self { angle }
    }

    pub fn from_direction(dir: Vec2) -> Self {
        Self {
            angle: dir.y.atan2(dir.x),
        }
    }

    /// Retourne la direction normalisée correspondant à l'angle
    pub fn to_direction(&self) -> Vec2 {
        Vec2 {
            x: self.angle.cos(),
            y: self.angle.sin(),
        }
    }
}

/// Vélocité (vitesse + direction)
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Vec2 {
        let len = self.length();
        if len > 0.0 {
            Vec2 {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            Vec2 { x: 0.0, y: 0.0 }
        }
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Self::zero()
    }
}

// ============================================================================
// GAMEPLAY COMPONENTS
// ============================================================================

/// Statistiques de combat
#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Stats {
    // Hull (coque) - Current / Max
    pub hull_current: f32,
    pub hull_max: f32,

    // Shield (bouclier) - Current / Max
    pub shield_current: f32,
    pub shield_max: f32,

    // Energy (énergie) - Current / Max
    pub energy_current: f32,
    pub energy_max: f32,

    // Régénération par seconde
    pub hull_regen: f32,
    pub shield_regen: f32,
    pub energy_regen: f32,
}

impl Stats {
    pub fn new(hull: f32, shield: f32, energy: f32) -> Self {
        Self {
            hull_current: hull,
            hull_max: hull,
            shield_current: shield,
            shield_max: shield,
            energy_current: energy,
            energy_max: energy,
            hull_regen: 0.0,
            shield_regen: 5.0,
            energy_regen: 10.0,
        }
    }

    pub fn hull_percent(&self) -> f32 {
        (self.hull_current / self.hull_max).clamp(0.0, 1.0)
    }

    pub fn shield_percent(&self) -> f32 {
        (self.shield_current / self.shield_max).clamp(0.0, 1.0)
    }

    pub fn energy_percent(&self) -> f32 {
        (self.energy_current / self.energy_max).clamp(0.0, 1.0)
    }

    pub fn is_alive(&self) -> bool {
        self.hull_current > 0.0
    }

    pub fn take_damage(&mut self, amount: f32) -> f32 {
        // D'abord les shields, puis la coque
        let shield_damage = amount.min(self.shield_current);
        self.shield_current -= shield_damage;

        let remaining = amount - shield_damage;
        let hull_damage = remaining.min(self.hull_current);
        self.hull_current -= hull_damage;

        shield_damage + hull_damage
    }

    pub fn regen(&mut self, delta: f32) {
        self.shield_current = (self.shield_current + self.shield_regen * delta)
            .min(self.shield_max);
        self.energy_current = (self.energy_current + self.energy_regen * delta)
            .min(self.energy_max);
        self.hull_current = (self.hull_current + self.hull_regen * delta)
            .min(self.hull_max);
    }
}

/// Données du vaisseau
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct ShipData {
    pub ship_class: ShipClass,
    pub tier: u8,

    // Stats de base
    pub base_speed: f32,
    pub acceleration: f32,
    pub turn_rate: f32,

    // Capacités
    pub cargo_capacity: u16,
    pub weapon_slots: u8,
    pub utility_slots: u8,
    pub passive_slots: u8,
}

impl ShipData {
    pub fn fighter() -> Self {
        Self {
            ship_class: ShipClass::Fighter,
            tier: 1,
            base_speed: 200.0,
            acceleration: 100.0,
            turn_rate: 3.0,
            cargo_capacity: 50,
            weapon_slots: 2,
            utility_slots: 2,
            passive_slots: 1,
        }
    }

    pub fn frigate() -> Self {
        Self {
            ship_class: ShipClass::Frigate,
            tier: 1,
            base_speed: 150.0,
            acceleration: 80.0,
            turn_rate: 2.0,
            cargo_capacity: 100,
            weapon_slots: 3,
            utility_slots: 2,
            passive_slots: 2,
        }
    }

    pub fn cruiser() -> Self {
        Self {
            ship_class: ShipClass::Cruiser,
            tier: 1,
            base_speed: 100.0,
            acceleration: 50.0,
            turn_rate: 1.0,
            cargo_capacity: 200,
            weapon_slots: 4,
            utility_slots: 3,
            passive_slots: 3,
        }
    }

    pub fn hauler() -> Self {
        Self {
            ship_class: ShipClass::Hauler,
            tier: 1,
            base_speed: 80.0,
            acceleration: 30.0,
            turn_rate: 0.8,
            cargo_capacity: 500,
            weapon_slots: 1,
            utility_slots: 2,
            passive_slots: 2,
        }
    }
}

/// État de combat
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct CombatState {
    /// Cible verrouillée
    pub target: Option<u64>,
    /// En combat (si true, pas de docking/désconnexion)
    pub in_combat: bool,
    /// Timer avant sortie de combat (secondes)
    pub combat_timer: f32,
}

impl CombatState {
    pub fn new() -> Self {
        Self {
            target: None,
            in_combat: false,
            combat_timer: 0.0,
        }
    }

    pub fn enter_combat(&mut self) {
        self.in_combat = true;
        self.combat_timer = 15.0; // 15s avant sortie
    }

    pub fn update(&mut self, delta: f32) {
        if self.in_combat {
            self.combat_timer -= delta;
            if self.combat_timer <= 0.0 {
                self.in_combat = false;
                self.target = None;
            }
        }
    }

    pub fn reset_timer(&mut self) {
        self.combat_timer = 15.0;
    }
}

impl Default for CombatState {
    fn default() -> Self {
        Self::new()
    }
}

/// Type d'entité (pour le rendu et la logique)
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    Player,
    NpcHostile,
    NpcFriendly,
    Asteroid,
    Station,
    Loot,
    Projectile,
    Effect,
}

// ============================================================================
// MARKER COMPONENTS
// ============================================================================

/// Marqueur pour le joueur local
#[derive(Component, Clone, Copy, Debug)]
pub struct LocalPlayer;

/// Marqueur pour une entité contrôlée par le serveur
#[derive(Component, Clone, Copy, Debug)]
pub struct ServerControlled;
