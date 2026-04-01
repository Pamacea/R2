//! Protocole de messages réseau R2
//!
//! Tous les messages échangés entre client et serveur via WebSocket.

use serde::{Deserialize, Serialize};
use crate::types::*;

// ============================================================================
// MESSAGE PRINCIPAL
// ============================================================================

/// Message principal R2 - échangé via WebSocket
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum R2Message {
    // ==================== CLIENT → SERVEUR ====================

    /// Mouvement vers une position
    MoveTo { target: Vec2 },

    /// Activation d'une compétence
    UseSkill {
        slot: u8,
        target_id: Option<u64>,
    },

    /// Interaction avec une entité
    Interact { entity_id: u64 },

    /// Message de chat
    Chat {
        channel: ChatChannel,
        message: String,
    },

    /// Requête marché
    MarketRequest {
        action: MarketAction,
        item_id: Option<u32>,
        price: Option<u64>,
        quantity: Option<u32>,
    },

    // ==================== SERVEUR → CLIENT ====================

    /// Snapshot complet du monde (initial sync)
    WorldSnapshot {
        entities: Vec<EntitySnapshot>,
        player_id: u64,
    },

    /// Mise à jour différentielle (par tick)
    WorldUpdate {
        tick: u64,
        updates: Vec<EntityUpdate>,
        deletes: Vec<u64>,
    },

    /// Événement de combat
    CombatEvent {
        source: u64,
        target: u64,
        damage: f32,
        damage_type: DamageType,
        vfx: VfxType,
    },

    /// Notification système
    SystemNotification {
        level: NotificationLevel,
        message: String,
    },

    /// Mise à jour de l'UI
    UIUpdate {
        component: UIComponent,
        data: UIData,
    },

    /// Erreur
    Error {
        code: ErrorCode,
        message: String,
    },

    // ==================== BIDIRECTIONNEL ====================

    /// Heartbeat/Ping
    Ping { timestamp: u64 },

    /// Pong response
    Pong { timestamp: u64 },
}

// ============================================================================
// SNAPSHOTS & UPDATES
// ============================================================================

/// Snapshot complet d'une entité (pour spawn initial)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EntitySnapshot {
    pub id: u64,
    pub archetype: EntityArchetype,
    pub position: (f32, f32, f32),
    pub rotation: f32,
    pub components: ComponentSnapshot,
}

/// Update différentiel (par tick)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EntityUpdate {
    pub id: u64,
    pub position: Option<(f32, f32, f32)>,
    pub rotation: Option<f32>,
    pub velocity: Option<(f32, f32)>,
    pub stats_delta: Option<StatsDelta>,
}

/// Delta de stats (pour optimisation)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StatsDelta {
    pub hull_delta: f32,
    pub shield_delta: f32,
    pub energy_delta: f32,
}

/// Snapshot des composants d'une entité
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ComponentSnapshot {
    pub stats: Option<Stats>,
    pub ship: Option<ShipClass>,
    pub combat: Option<CombatSnapshot>,
    pub name: Option<String>,
    pub faction: Option<Faction>,
}

/// Snapshot de l'état de combat
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CombatSnapshot {
    pub target: Option<u64>,
    pub in_combat: bool,
}

// ============================================================================
// UI DATA
// ============================================================================

/// Composant UI à mettre à jour
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UIComponent {
    HealthBar,
    ShieldBar,
    EnergyBar,
    SkillBar(u8), // 0-7
    Inventory,
    Minimap,
    Chat,
    TargetInfo,
    Market,
}

/// Données UI
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UIData {
    Stat { current: f32, max: f32 },
    Skill { ready: bool, cooldown: f32 },
    Inventory { items: Vec<ItemData> },
    Target { name: String, hp_percent: f32 },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ItemData {
    pub id: u32,
    pub name: String,
    pub quantity: u32,
    pub rarity: ItemRarity,
}

// ============================================================================
// TYPES D'ENTITÉS
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityArchetype {
    Player,
    Npc,
    Asteroid,
    Station,
    LootContainer,
    Projectile,
    Effect,
}

// ============================================================================
// CHAT
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatChannel {
    Local,     // Proximity
    System,    // Server messages
    Guild,
    Private,
    Trade,
}

// ============================================================================
// MARKET
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketAction {
    ListOrders,
    Buy,
    Sell,
    CancelOrder,
}

// ============================================================================
// COMBAT
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Kinetic,
    Thermal,
    EM,
    Explosive,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VfxType {
    LaserHit,
    Explosion,
    MissileTrail,
    ShieldHit,
}

// ============================================================================
// NOTIFICATIONS
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Achievement,
}

// ============================================================================
// ERREURS
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidMove,
    Cooldown,
    OutOfRange,
    OutOfEnergy,
    InvalidTarget,
    Unauthorized,
    RateLimited,
    ServerError,
}
