# Networking R2 - Protocole de Communication

> **Version** : 0.1.0 | **Dernière mise à jour** : 2025-04-01

---

## 📡 Vue d'Ensemble

R2 utilise **WebSocket** pour la communication temps réel entre le client et le serveur. Le protocole est **type-safe** grâce à Rust/Serde : tous les messages sont définis dans `r2_core` et partagés entre client et serveur.

```
┌─────────────────────────────────────────────────────────────────┐
│                    PROTOCOLE R2                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌───────────────┐              ┌───────────────┐              │
│  │     Client    │◄────────────►│    Serveur    │              │
│  │   (Browser)   │   WebSocket   │  (Cluster)    │              │
│  └───────────────┘              └───────────────┘              │
│        │                                │                       │
│        │                                │                       │
│   Client→Server                    Server→Client                 │
│   - Input                          - WorldState                 │
│   - Actions                        - EntityUpdates              │
│   - Requests                       - Events                     │
│                                    - Errors                     │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🔄 Flux de Connexion

```
┌─────────┐       ┌──────────────┐       ┌──────────────┐
│ Client  │       │ API Gateway  │       │ Game Cluster │
└────┬────┘       └──────┬───────┘       └──────┬───────┘
     │                   │                       │
     │ POST /auth/oauth/google│                   │
     ├──────────────────►│                       │
     │                   │                       │
     │                   │ OAuth Redirect        │
     │◄──────────────────┤                       │
     │                   │                       │
     │ POST /auth/callback                       │
     ├──────────────────►│                       │
     │                   │                       │
     │ JWT + Refresh     │                       │
     │◄──────────────────┤                       │
     │                   │                       │
     │ WS /ws/connect    │                       │
     │ (with JWT)        │                       │
     ├──────────────────►│                       │
     │                   │                       │
     │                   │ Route to cluster      │
     │                   ├──────────────────────►│
     │                   │                       │
     │ WS Upgrade OK     │                       │
     │◄──────────────────┤                       │
     │                   │                       │
     │───── R2Message ──────────────────────────►│
     │◄──── R2Message ───────────────────────────┤
     │                   │                       │
```

---

## 📦 Messages - Définition Complète

Tous les messages sont définis dans `r2_core/src/network/msgs.rs`.

### Enum Principale

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum R2Message {
    // ========== CLIENT → SERVEUR ==========

    /// Mouvement vers une position
    MoveTo { target: Vec2 },

    /// Activation d'une compétence
    UseSkill {
        slot: u8,
        target_id: Option<EntityId>,
    },

    /// Interaction avec une entité
    Interact { entity_id: EntityId },

    /// Chat message
    Chat { channel: ChatChannel, message: String },

    /// Request marché
    MarketRequest {
        action: MarketAction,
        item_id: Option<ItemId>,
        price: Option<u64>,
        quantity: Option<u32>,
    },

    // ========== SERVEUR → CLIENT ==========

    /// Snapshot complet du monde (initial sync)
    WorldSnapshot {
        entities: Vec<EntitySnapshot>,
        player_id: EntityId,
    },

    /// Mise à jour différentielle (tick)
    WorldUpdate {
        tick: u64,
        updates: Vec<EntityUpdate>,
        deletes: Vec<EntityId>,
    },

    /// Événement de combat
    CombatEvent {
        source: EntityId,
        target: EntityId,
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

    // ========== bidirectionnel ==========

    /// Heartbeat/Ping
    Ping { timestamp: u64 },

    /// Pong response
    Pong { timestamp: u64 },
}
```

---

## 🎮 Types de Données

### EntitySnapshot

```rust
/// Snapshot complet d'une entité (pour spawn initial)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EntitySnapshot {
    pub id: EntityId,
    pub archetype: EntityArchetype,

    pub position: Vec3,
    pub rotation: Quat,

    pub components: ComponentSnapshot,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComponentSnapshot {
    pub stats: Option<Stats>,
    pub ship: Option<ShipData>,
    pub combat: Option<CombatState>,
    pub name: Option<String>,
    pub faction: Option<Faction>,
}
```

### EntityUpdate

```rust
/// Update différentiel (par tick)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EntityUpdate {
    pub id: EntityId,

    // Seuls les champs modifiés sont envoyés
    pub position: Option<Vec3>,
    pub rotation: Option<Quat>,
    pub velocity: Option<Vec3>,

    pub stats_delta: Option<StatsDelta>, // +HP, -Energy, etc
}
```

### Types Métier

```rust
pub type EntityId = u64;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum EntityArchetype {
    Player,
    Npc,
    Asteroid,
    Station,
    LootContainer,
    Projectile,
    Effect,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ChatChannel {
    Local,     // Proximity
    System,    // Server messages
    Guild,
    Private,
    Trade,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MarketAction {
    ListOrders,
    Buy,
    Sell,
    CancelOrder,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DamageType {
    Kinetic,
    Thermal,
    EM,
    Explosive,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VfxType {
    LaserHit,
    Explosion,
    MissileTrail,
    ShieldHit,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Achievement,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ErrorCode {
    InvalidMove,
    Cooldown,
    OutOfRange,
    OutOfEnergy,
    InvalidTarget,
    Unauthorized,
}
```

---

## ⚡ Optimisations Réseau

### 1. Compression

Tous les messages sont compressés avec **zstd** avant envoi.

```rust
// Serialization avec compression
pub fn serialize_msg(msg: &R2Message) -> Result<Vec<u8>> {
    let json = serde_json::to_vec(msg)?;
    let compressed = zstd::encode_all(&*json, 3)?;
    Ok(compressed)
}
```

### 2. Delta Encoding

Seuls les changements sont envoyés :

```rust
// Au lieu d'envoyer toute la position
// Vec3 { x: 1234.567, y: 2345.678, z: 0.0 }

// On envoie le delta relatif au tick précédent
// PositionDelta { dx: 0.5, dy: 0.3, dz: 0.0 }
```

### 3. Quantification

Les positions sont quantifiées en **millimètres** (pas f32) :

```rust
// f32 position (4 bytes × 3 = 12 bytes)
Vec3 { x: 1234.567, y: 2345.678, z: 0.0 }

// → Quantized (4 bytes × 3 = 12 bytes, mais plus compressible)
PositionQuantized { x: 1234567, y: 2345678, z: 0 }  // i32 en mm
```

### 4. Interest Management

```rust
// Spatial filtering côté serveur
pub fn get_relevant_entities(
    player: &Player,
    world: &World,
    spatial_index: &SpatialHashGrid,
) -> Vec<EntityId> {
    // Entités dans la grille du joueur + 8 adjacentes
    // + entités marquées "always_relevant" (joueur, party members)
}
```

---

## 🔐 Sécurité

### Validation Côté Serveur

```rust
// Exemple : validation mouvement
pub fn validate_movement(
    player_id: EntityId,
    current_pos: Vec3,
    target: Vec2,
    world: &World,
) -> Result<(), MovementError> {
    // Vérifier que le mouvement est possible
    let distance = current_pos.truncate().distance(target);

    // Vitesse max du vaisseau
    let max_speed = world.get_max_speed(player_id);

    // Distance max par tick (anti-speedhack)
    let max_delta = max_speed * TICK_DURATION;

    if distance > max_delta {
        return Err(MovementError::TooFast);
    }

    // Vérifier que la destination est dans la map
    if !world.is_valid_position(target) {
        return Err(MovementError::OutOfBounds);
    }

    Ok(())
}
```

### Rate Limiting

```rust
// Limiter les actions par joueur
pub struct RateLimiter {
    move_budget: f32,      // Recharge over time
    skill_budget: f32,
}

// Si le joueur spam, son budget est épuisé
// → Actions ignorées ou pénalisées
```

---

## 📊 Taille des Messages

| Message | Taille typique (compressé) | Fréquence |
|---------|---------------------------|-----------|
| MoveTo | ~8 bytes | Par input joueur |
| WorldUpdate | ~500 bytes - 2KB | 20-30 Hz |
| CombatEvent | ~32 bytes | Event-driven |
| Ping/Pong | ~12 bytes | 1 Hz |

**Bandwidth par joueur estimé :** 5-15 KB/s (variable selon densité)

---

## 🧪 Tests de Connexion

```bash
# Test WebSocket avec wscat
wscat -c "wss://dev.r2-game.com/ws/connect" \
  -H "Authorization: Bearer <JWT>"

# Envoyer un message
{"MoveTo":{"target":{"x":100.0,"y":200.0}}}
```

---

*Networking v0.1.0 - Protocole sujet à optimisation*
