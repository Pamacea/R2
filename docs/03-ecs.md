# ECS Design R2

> **Version** : 0.1.0 | **Dernière mise à jour** : 2025-04-01

---

## 🧩 Philosophie ECS

R2 utilise **Bevy ECS** pour le client et **Bevy headless** pour le serveur. Cela permet :
- **Code partagé** : Mêmes composants, mêmes systèmes (presque)
- **Performance** : Cache-friendly, parallélisable
- **Flexibilité** : Ajouter/modifier des features sans refactor

> **Règle d'or** : Data-Oriented, pas Object-Oriented.

---

## 📦 Composants (Les Données)

Les composants sont définis dans `r2_core/src/components/`.

### Core Components

```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Identifiant unique d'entité (répliqué depuis le serveur)
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct NetworkId(pub u64);

/// Position dans le monde
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Rotation
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Rotation {
    pub angle: f32, // En radians, autour de Z
}

/// Vélocité (mouvement)
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

/// Échelle visuelle
#[derive(Component, Clone, Debug)]
pub struct Scale {
    pub value: f32,
}
```

### Gameplay Components

```rust
/// Statistiques de combat
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Stats {
    // Current / Max
    pub hull: (f32, f32),       // Coque (HP)
    pub shield: (f32, f32),     // Bouclier
    pub energy: (f32, f32),     // Énergie
    pub capacitor: (f32, f32),  // Capacitor (boost speed)

    // Régénération par seconde
    pub hull_regen: f32,
    pub shield_regen: f32,
    pub energy_regen: f32,
}

/// Données du vaisseau
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct ShipData {
    pub ship_class: ShipClass,
    pub base_speed: f32,
    pub acceleration: f32,
    pub turn_rate: f32,

    // Slots d'équipement
    pub weapon_slots: u8,
    pub utility_slots: u8,
    pub passive_slots: u8,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum ShipClass {
    Fighter,    // Rapide, fragile
    Frigate,    // Équilibré
    Cruiser,    // Lent, tanky
    Hauler,     // Cargo, peu d'armes
}

/// Inventaire
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<Option<ItemStack>>,
    pub capacity: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemStack {
    pub item_id: u32,
    pub quantity: u32,
    pub quality: ItemQuality,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum ItemQuality {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}
```

### Combat Components

```rust
/// État de combat
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct CombatState {
    pub target: Option<Entity>,   // Cible verrouillée
    pub in_combat: bool,
    pub combat_timer: f32,        // Temps avant sortie de combat
}

/// Cooldowns de compétences
#[derive(Component, Clone, Debug)]
pub struct Cooldowns {
    pub slots: [f32; 8], // cooldown restant par slot (0 = ready)
}

/// Tags de faction
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Faction {
    pub id: u8,
    pub friendly_fire: bool,
}

/// Type d'entité
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
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
```

### AI Components (Serveur uniquement)

```rust
/// Comportement d'IA
#[derive(Component, Clone, Debug)]
pub struct AIBehavior {
    pub behavior_type: AIBehaviorType,
    pub state: AIState,
    pub home_position: Vec3,
    pub patrol_radius: f32,
    pub aggro_range: f32,
    pub leash_range: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AIBehaviorType {
    Passive,       // Fuit si attaqué
    Neutral,       // Attaque si proche
    Aggressive,    // Attaque à vue
    Guardian,      // Protège une zone
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AIState {
    Idle,
    Patrol,
    Chase,
    Attack,
    Return,
    Flee,
}
```

### Client-Only Components

```rust
/// Animation sprite/mesh
#[derive(Component, Clone, Debug)]
pub struct Animation {
    pub state: AnimationState,
    pub timer: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnimationState {
    Idle,
    Move,
    Attack,
    Hit,
    Destroyed,
}

/// Trail des projectiles
#[derive(Component, Clone, Debug)]
pub struct Trail {
    pub lifetime: f32,
    pub points: Vec<Vec3>,
}
```

---

## ⚙️ Systèmes (La Logique)

Les systèmes sont séparés en 3 catégories :
1. **Shared** : Fonctionnent client ET serveur
2. **Server** : Simulation uniquement
3. **Client** : Rendu/prédiction uniquement

### Movement Systems

```rust
// === SHARED ===
/// Applique la vélocité à la position
pub fn apply_velocity_system(
    mut query: Query<(&Velocity, &mut Position)>,
    time: Res<Time>,
) {
    for (velocity, mut position) in query.iter_mut() {
        position.x += velocity.x * time.delta_seconds();
        position.y += velocity.y * time.delta_seconds();
        position.z += velocity.z * time.delta_seconds();
    }
}

// === CLIENT ===
/// Mouvement vers une cible (client-side prediction)
pub fn move_to_target_system(
    mut query: Query<(
        &MoveTarget,
        &ShipData,
        &mut Velocity,
        &mut Rotation,
    )>,
    time: Res<Time>,
) {
    for (target, ship, mut velocity, mut rotation) in query.iter_mut() {
        let direction = target.position - current_position;
        let distance = direction.length();

        if distance > 5.0 {
            // Rotation vers la cible
            let target_angle = direction.y.atan2(direction.x);
            rotation.angle = target_angle;

            // Accélération
            let speed = ship.base_speed;
            velocity.x = direction.normalize().x * speed;
            velocity.y = direction.normalize().y * speed;
        } else {
            // Arrivé
            *velocity = Velocity::default();
        }
    }
}

// === SERVER ===
/// Validation du mouvement + vérification collision
pub fn validate_movement_system(
    mut move_events: EventReader<MoveEvent>,
    mut query: Query<(&NetworkId, &mut Position)>,
    spatial_grid: Res<SpatialHashGrid>,
) {
    for event in move_events.iter() {
        // Vérifier que le mouvement est légal
        if validate_move(event) {
            // Appliquer
            if let Ok(mut position) = query.get_mut(event.entity) {
                position.x = event.new_x;
                position.y = event.new_y;
            }
        } else {
            // Corriger le client
            send_correction(event.player_id, position);
        }
    }
}
```

### Combat Systems

```rust
// === SHARED ===
/// Régénération des stats
pub fn regen_system(
    mut query: Query<&mut Stats>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();

    for mut stats in query.iter_mut() {
        stats.hull.0 = (stats.hull.0 + stats.hull_regen * delta).min(stats.hull.1);
        stats.shield.0 = (stats.shield.0 + stats.shield_regen * delta).min(stats.shield.1);
        stats.energy.0 = (stats.energy.0 + stats.energy_regen * delta).min(stats.energy.1);
    }
}

// === SERVER ===
/// Application des dégâts
pub fn apply_damage_system(
    mut commands: Commands,
    mut damage_events: EventReader<DamageEvent>,
    mut query: Query<(&mut Stats, &mut CombatState)>,
) {
    for event in damage_events.iter() {
        if let Ok((mut stats, mut combat)) = query.get_mut(event.target) {
            // Appliquer dégâts bouclier puis coque
            let remaining = apply_damage_to_shield(&mut stats, event.damage);
            apply_damage_to_hull(&mut stats, remaining);

            // Marquer en combat
            combat.in_combat = true;
            combat.combat_timer = 15.0;

            // Notification client
            send_combat_event(event);

            // Check mort
            if stats.hull.0 <= 0.0 {
                commands.entity(event.target).despawn();
                spawn_loot(event.target);
            }
        }
    }
}
```

### AI Systems

```rust
// === SERVER ONLY ===
pub fn ai_state_machine_system(
    mut query: Query<(
        Entity,
        &AIBehavior,
        &mut Velocity,
        &Position,
        &mut CombatState,
    )>,
    players: Query<(Entity, &Position, &Faction)>,
) {
    for (entity, ai, mut velocity, position, mut combat) in query.iter_mut() {
        match ai.state {
            AIState::Idle => {
                // Chercher joueur proche
                if let Some(target) = find_target(position, &players, ai.aggro_range) {
                    combat.target = Some(target);
                    ai.state = AIState::Chase;
                }
            }
            AIState::Chase => {
                // Se rapprocher de la cible
                if let Some(target) = combat.target {
                    if let Ok(target_pos) = players.get_component::<Position>(target) {
                        move_towards(position, target_pos, &mut velocity);
                    }
                }
            }
            AIState::Attack => {
                // Attaquer si à portée
                if in_range(position, combat.target, WEAPON_RANGE) {
                    // Tirer
                } else {
                    ai.state = AIState::Chase;
                }
            }
            _ => {}
        }
    }
}
```

---

## 🎯 Systèmes Sets (Ordre d'exécution)

```rust
// Dans r2_client et r2_server
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSet {
    // 1. Input handling (client) / Network events (server)
    Input,

    // 2. Simulation
    Movement,
    Combat,
    AI,

    // 3. World updates
    Regeneration,
    Cooldowns,

    // 4. Cleanup
    Despawn,

    // 5. Network sync (server) / Rendering (client)
    Sync,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)

        // Systèmes dans l'ordre
        .add_systems(
            Update,
            (
                input_system.in_set(GameSet::Input),
                move_to_target_system.in_set(GameSet::Movement),
                apply_damage_system.in_set(GameSet::Combat),
                regen_system.in_set(GameSet::Regeneration),
            )
            .chain() // Exécution dans l'ordre
        )
        .run();
}
```

---

## 🔄 Réplication Client ↔ Serveur

```
┌─────────────────────────────────────────────────────────────────┐
│                     RÉPLICATION R2                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   SERVEUR (Authorité)               CLIENT (Prédiction)         │
│   ┌─────────────┐                  ┌─────────────┐              │
│   │   ECS       │                  │   ECS       │              │
│   │  Position   │◄──── sync ──────►│  Position   │              │
│   │  Velocity   │                  │  Velocity   │              │
│   │  Stats      │                  │  Stats      │              │
│   └─────────────┘                  └─────────────┘              │
│         │                                │                       │
│         │ 1. WorldUpdate (20Hz)         │                       │
│         ├──────────────────────────────►│                       │
│         │                                │                       │
│         │                                │ 2. Interpolation      │
│         │                                │    (smooth)           │
│         │                                │                       │
│         │                                │ 3. Render            │
│         │                                │                       │
│   CLIENT                           SERVEUR                      │
│   ┌─────────────┐                  ┌─────────────┐              │
│   │  Input      │                  │ Validation  │              │
│   └─────────────┘                  └─────────────┘              │
│         │                                │                       │
│         │ MoveTo request                 │                       │
│         ├──────────────────────────────►│                       │
│         │                                │                       │
│         │                                │ 4. Apply if valid     │
│         │                                │                       │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📊 Performance Tips

### 1. Query Optimization

```rust
// ❌ Mauvais - trop de composants
fn bad_system(query: Query<(&A, &B, &C, &D, &E, &F)>)

// ✅ Bon - splitter en systèmes
fn good_system_1(query: Query<(&A, &B)>)
fn good_system_2(query: Query<(&C, &D)>)
```

### 2. Archetypes

```rust
// Regrouper les entités similaires
// - Toutes les entités Position + Velocity → même archetype
// → Cache-friendly
```

### 3. Parallel Systems

```rust
// Les systèmes sans conflits peuvent tourner en parallèle
app.add_systems(
    Update,
    (
        regen_system,
        cooldown_system,
        animation_system,
    ).into_iter().chain(), // Parallèle
)
```

---

*ECS Design v0.1.0 - Architecture évolutive*
