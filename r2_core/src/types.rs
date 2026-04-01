//! Types de domaine R2
//!
//! Types utilisés dans tout le projet.

use serde::{Deserialize, Serialize};

// ============================================================================
// TYPES PRIMITIFS
// ============================================================================

/// Vecteur 2D
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn distance(&self, other: &Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            Self::ZERO
        }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

// ============================================================================
// SHIPS
// ============================================================================

/// Classe de vaisseau
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipClass {
    Fighter,    // Rapide, fragile
    Frigate,    // Équilibré
    Cruiser,    // Lent, tanky
    Hauler,     // Cargo, peu d'armes
}

impl ShipClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fighter => "fighter",
            Self::Frigate => "frigate",
            Self::Cruiser => "cruiser",
            Self::Hauler => "hauler",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fighter" => Some(Self::Fighter),
            "frigate" => Some(Self::Frigate),
            "cruiser" => Some(Self::Cruiser),
            "hauler" => Some(Self::Hauler),
            _ => None,
        }
    }
}

// ============================================================================
// FACTIONS
// ============================================================================

/// Faction dans le jeu
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Faction {
    Neutral,
    Terran,
    Martian,
    Beltalian,
    Pirate,
    Corporation,
}

impl Faction {
    pub fn id(&self) -> u8 {
        match self {
            Self::Neutral => 0,
            Self::Terran => 1,
            Self::Martian => 2,
            Self::Beltalian => 3,
            Self::Pirate => 4,
            Self::Corporation => 5,
        }
    }

    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::Neutral),
            1 => Some(Self::Terran),
            2 => Some(Self::Martian),
            3 => Some(Self::Beltalian),
            4 => Some(Self::Pirate),
            5 => Some(Self::Corporation),
            _ => None,
        }
    }

    /// Est-ce que cette faction est hostile à l'autre ?
    pub fn is_hostile_to(&self, other: &Self) -> bool {
        match (self, other) {
            // Les pirates sont hostiles à tout le monde sauf eux-mêmes
            (Self::Pirate, Self::Pirate) => false,
            (Self::Pirate, _) | (_, Self::Pirate) => true,

            // Les factions majeures entre elles (guerre froide)
            (Self::Terran, Self::Martian) | (Self::Martian, Self::Terran) => true,

            // Beltalian vs Terran (conflit minier)
            (Self::Beltalian, Self::Terran) | (Self::Terran, Self::Beltalian) => true,

            _ => false,
        }
    }
}

// ============================================================================
// ITEMS
// ============================================================================

/// Rareté d'item
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl ItemRarity {
    pub fn color(&self) -> &'static str {
        match self {
            Self::Common => "#8b949e",
            Self::Uncommon => "#3fb950",
            Self::Rare => "#58a6ff",
            Self::Epic => "#d2a8ff",
            Self::Legendary => "#f0e68c",
        }
    }
}

/// Type d'item
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    Weapon,
    Utility,
    Passive,
    Resource,
    Module,
    Consumable,
}

// ============================================================================
// RE-EXPORTS DES COMPOSANTS (utilisé par r2_core/src/lib.rs)
// ============================================================================

// NOTE: Stats, ShipData, etc. sont définis dans components.rs
// Ce fichier contient seulement les types simples et enums

// Import pour components.rs
pub use crate::components::{
    CombatState, Position, Rotation, ShipData, ShipData as ShipDataStruct, Stats, Velocity,
};

// Re-export des structs de components pour utilisation externe
#[allow(clippy::enum_variant_names)]
pub enum ComponentType {
    Stats,
    ShipData,
    CombatState,
    Position,
    Rotation,
    Velocity,
}

// ============================================================================
// CONSTANTS
// ============================================================================

/// Tick rate du serveur (Hz)
pub const SERVER_TICK_RATE: u32 = 30;

/// Intervalle entre ticks (secondes)
pub const SERVER_TICK_INTERVAL: f32 = 1.0 / SERVER_TICK_RATE as f32;

/// Network update rate (Hz)
pub const NETWORK_UPDATE_RATE: u32 = 20;

/// Port par défaut du serveur de jeu
pub const DEFAULT_GAME_PORT: u16 = 3000;

/// Port par défaut de l'API Gateway
pub const DEFAULT_API_PORT: u16 = 8080;

// ============================================================================
// ZONES
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneType {
    Safe,    // Pas de PvP
    PvP,     // Full loot
    PVE,     // Ennemis seulement
    Arena,   // Matchmaking
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneId(pub u16);

impl ZoneId {
    pub const START: Self = Self(1);
    pub const SAFE_ZONE_1: Self = Self(10);
    pub const PVP_ZONE_1: Self = Self(100);
}
