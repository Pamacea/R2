//! Composants ECS serveur-only
//!
//! Ces composants ne sont pas partagés avec le client.
//! Pour les composants partagés, voir `r2_core::components`.

use bevy_ecs::component::Component;
use r2_core::{Position, ShipClass};

// ============================================================================
// PLAYER COMPONENTS
// ============================================================================

/// Identifiant du joueur (de la base de données)
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayerId(pub i64);

impl PlayerId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    pub fn is_valid(&self) -> bool {
        self.0 > 0
    }
}

/// Données du joueur connecté
#[derive(Component, Clone, Debug)]
pub struct Player {
    /// ID du joueur (base de données)
    pub id: i64,
    /// ID du personnage actif
    pub character_id: i64,
    /// Nom du personnage
    pub name: String,
    /// Niveau actuel
    pub level: u8,
    /// Expérience actuelle
    pub exp: u64,
    /// Faction du joueur
    pub faction: r2_core::Faction,
    /// Timestamp de dernière activité
    pub last_activity: std::time::Instant,
}

impl Player {
    pub fn new(id: i64, character_id: i64, name: String) -> Self {
        Self {
            id,
            character_id,
            name,
            level: 1,
            exp: 0,
            faction: r2_core::Faction::Neutral,
            last_activity: std::time::Instant::now(),
        }
    }

    /// Marquer une activité (pour timeout AFK)
    pub fn mark_activity(&mut self) {
        self.last_activity = std::time::Instant::now();
    }

    /// Vérifier si le joueur est AFK (> 5 minutes)
    pub fn is_afk(&self) -> bool {
        self.last_activity.elapsed() > std::time::Duration::from_secs(300)
    }
}

// ============================================================================
// SHIP COMPONENTS
// ============================================================================

/// Composant marquant une entité comme étant un vaisseau joueur
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerShip {
    /// Index du joueur propriétaire
    pub player_id: i64,
    /// Classe du vaisseau
    pub ship_class: ShipClass,
}

impl PlayerShip {
    pub fn new(player_id: i64, ship_class: ShipClass) -> Self {
        Self {
            player_id,
            ship_class,
        }
    }
}

/// État de propulsion du vaisseau
#[derive(Component, Clone, Copy, Debug)]
pub struct Propulsion {
    /// Vitesse maximale
    pub max_speed: f32,
    /// Accélération
    pub acceleration: f32,
    /// Vitesse de rotation (rad/s)
    pub turn_rate: f32,
    /// Vitesse actuelle (0-1)
    pub throttle: f32,
    /// Cible de mouvement (optionnelle)
    pub move_target: Option<(f32, f32)>,
}

impl Propulsion {
    pub fn new(max_speed: f32, acceleration: f32, turn_rate: f32) -> Self {
        Self {
            max_speed,
            acceleration,
            turn_rate,
            throttle: 0.0,
            move_target: None,
        }
    }

    /// Configurer selon la classe de vaisseau
    pub fn for_ship_class(ship_class: ShipClass) -> Self {
        match ship_class {
            ShipClass::Fighter => Self::new(200.0, 100.0, 3.0),
            ShipClass::Frigate => Self::new(150.0, 80.0, 2.0),
            ShipClass::Cruiser => Self::new(100.0, 50.0, 1.0),
            ShipClass::Hauler => Self::new(80.0, 30.0, 0.8),
        }
    }

    /// Définir la cible de mouvement
    pub fn set_move_target(&mut self, x: f32, y: f32) {
        self.move_target = Some((x, y));
    }

    /// Effacer la cible de mouvement
    pub fn clear_move_target(&mut self) {
        self.move_target = None;
        self.throttle = 0.0;
    }
}

// ============================================================================
// WEAPON COMPONENTS
// ============================================================================

/// Arme équipée sur un vaisseau
#[derive(Component, Clone, Debug)]
pub struct Weapon {
    /// Slot de l'arme (0-7)
    pub slot: u8,
    /// Dégâts de base
    pub damage: f32,
    /// Portée en unités monde
    pub range: f32,
    /// Vélocité des projectiles (0 = instantané/laser)
    pub projectile_speed: f32,
    /// Cooldown entre les tirs (secondes)
    pub cooldown: f32,
    /// Cooldown restant
    pub cooldown_remaining: f32,
    /// Type de dégâts
    pub damage_type: r2_core::DamageType,
    /// Coût en énergie
    pub energy_cost: f32,
}

impl Weapon {
    /// Créer une nouvelle arme
    pub fn new(
        slot: u8,
        damage: f32,
        range: f32,
        projectile_speed: f32,
        cooldown: f32,
        damage_type: r2_core::DamageType,
    ) -> Self {
        Self {
            slot,
            damage,
            range,
            projectile_speed,
            cooldown,
            cooldown_remaining: 0.0,
            damage_type,
            energy_cost: 10.0,
        }
    }

    /// Arme laser de base
    pub fn laser(slot: u8) -> Self {
        Self::new(slot, 10.0, 500.0, 0.0, 0.5, r2_core::DamageType::Thermal)
    }

    /// Arme à projectile
    pub fn projectile(slot: u8) -> Self {
        Self::new(slot, 25.0, 800.0, 400.0, 1.0, r2_core::DamageType::Kinetic)
    }

    /// Vérifier si l'arme est prête à tirer
    pub fn is_ready(&self) -> bool {
        self.cooldown_remaining <= 0.0
    }

    /// Tenter de tirer
    pub fn fire(&mut self) -> bool {
        if self.is_ready() {
            self.cooldown_remaining = self.cooldown;
            true
        } else {
            false
        }
    }

    /// Mettre à jour le cooldown
    pub fn update(&mut self, delta: f32) {
        self.cooldown_remaining = (self.cooldown_remaining - delta).max(0.0);
    }
}

/// Projectile tiré par une arme
#[derive(Component, Clone, Debug)]
pub struct Projectile {
    /// Entité qui a tiré
    pub owner: i64,
    /// Dégâts infligés
    pub damage: f32,
    /// Type de dégâts
    pub damage_type: r2_core::DamageType,
    /// Durée de vie restante
    pub lifetime: f32,
}

impl Projectile {
    pub fn new(owner: i64, damage: f32, damage_type: r2_core::DamageType) -> Self {
        Self {
            owner,
            damage,
            damage_type,
            lifetime: 5.0, // 5 secondes max
        }
    }
}

// ============================================================================
// COLLISION COMPONENTS
// ============================================================================

/// Sphère de collision pour les entités spatiales
#[derive(Component, Clone, Copy, Debug)]
pub struct CollisionSphere {
    /// Rayon de collision
    pub radius: f32,
    /// Type de collision
    pub collision_type: CollisionType,
    /// Masque de collision (bits)
    pub collision_mask: u32,
}

impl CollisionSphere {
    pub fn new(radius: f32, collision_type: CollisionType) -> Self {
        Self {
            radius,
            collision_type,
            collision_mask: collision_type.mask(),
        }
    }

    /// Rayon pour un vaisseau (selon sa classe)
    pub fn for_ship_class(ship_class: ShipClass) -> Self {
        let radius = match ship_class {
            ShipClass::Fighter => 10.0,
            ShipClass::Frigate => 15.0,
            ShipClass::Cruiser => 25.0,
            ShipClass::Hauler => 30.0,
        };
        Self::new(radius, CollisionType::Ship)
    }

    /// Rayon pour un projectile
    pub fn projectile() -> Self {
        Self::new(2.0, CollisionType::Projectile)
    }

    /// Vérifier si deux sphères collisionnent
    pub fn collides_with(
        &self,
        other_pos: &Position,
        my_pos: &Position,
        other: &CollisionSphere,
    ) -> bool {
        // Masques de collision
        if (self.collision_mask & other.collision_mask) == 0 {
            return false;
        }

        // Distance au carré
        let dx = other_pos.x - my_pos.x;
        let dy = other_pos.y - my_pos.y;
        let dist_sq = dx * dx + dy * dy;
        let radius_sum = self.radius + other.radius;

        dist_sq < radius_sum * radius_sum
    }
}

/// Type de collision pour les masques
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollisionType {
    Ship,
    Projectile,
    Station,
    Asteroid,
    Loot,
}

impl CollisionType {
    /// Masque de collision pour ce type
    pub fn mask(&self) -> u32 {
        match self {
            Self::Ship => 0b001,
            Self::Projectile => 0b010,
            Self::Station => 0b100,
            Self::Asteroid => 0b001,
            Self::Loot => 0b001,
        }
    }

    /// Ce type collisionne-t-il avec l'autre ?
    pub fn collides_with(&self, other: &Self) -> bool {
        match (self, other) {
            // Les projectiles touchent les vaisseaux
            (Self::Projectile, Self::Ship) | (Self::Ship, Self::Projectile) => true,

            // Les vaisseaux se touchent entre eux
            (Self::Ship, Self::Ship) => true,

            // Tout touche les stations
            (_, Self::Station) | (Self::Station, _) => true,

            // Les astéroïdes touchent les vaisseaux
            (Self::Asteroid, Self::Ship) | (Self::Ship, Self::Asteroid) => true,

            _ => false,
        }
    }
}

// ============================================================================
// AI COMPONENTS
// ============================================================================

/// Comportement d'une entité contrôlée par l'IA
#[derive(Component, Clone, Debug)]
pub struct AiBehavior {
    /// Type d'IA
    pub behavior_type: AiType,
    /// Cible actuelle
    pub target: Option<u64>,
    /// État de patrol
    pub patrol_center: (f32, f32),
    pub patrol_radius: f32,
    /// Timer pour les décisions
    pub decision_timer: f32,
}

impl AiBehavior {
    pub fn new(behavior_type: AiType, center: (f32, f32), radius: f32) -> Self {
        Self {
            behavior_type,
            target: None,
            patrol_center: center,
            patrol_radius: radius,
            decision_timer: 0.0,
        }
    }

    /// IA hostile (pirate, etc.)
    pub fn hostile(center: (f32, f32)) -> Self {
        Self::new(AiType::Hostile, center, 500.0)
    }

    /// IA passive (marchand, station)
    pub fn passive(center: (f32, f32)) -> Self {
        Self::new(AiType::Passive, center, 100.0)
    }
}

/// Type de comportement IA
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiType {
    /// Ne bouge pas, réagit si attaqué
    Passive,
    /// Patrouille, attaque si proche
    Patrol,
    /// Hostile, attaque à vue
    Hostile,
    /// Fuit si attaqué
    Coward,
}

// ============================================================================
// MARKER COMPONENTS
// ============================================================================

/// Marqueur pour une entité qui doit être synchronisée avec les clients
#[derive(Component, Clone, Copy, Debug)]
pub struct NetworkSync {
    /// Flag dirty pour forcer l'update
    pub dirty: bool,
}

impl NetworkSync {
    pub fn new() -> Self {
        Self { dirty: true }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

impl Default for NetworkSync {
    fn default() -> Self {
        Self::new()
    }
}

/// Marqueur pour une entité qui va être supprimée (despawn)
#[derive(Component, Clone, Copy, Debug)]
pub struct DespawnMarker;

// ============================================================================
// SPAWNER COMPONENTS
// ============================================================================

/// Spawner d'entités (pour les NPCs, astéroïdes, etc.)
#[derive(Component, Clone, Debug)]
pub struct Spawner {
    /// Type d'entité à spawner
    pub spawn_type: SpawnType,
    /// Intervalle entre les spawns
    pub interval: f32,
    /// Timer restant
    pub timer: f32,
    /// Nombre max d'entités
    pub max_count: usize,
    /// Rayon autour du spawner
    pub radius: f32,
}

impl Spawner {
    pub fn new(spawn_type: SpawnType, interval: f32, max_count: usize, radius: f32) -> Self {
        Self {
            spawn_type,
            interval,
            timer: 0.0,
            max_count,
            radius,
        }
    }

    /// Mettre à jour et retourner true si on doit spawner
    pub fn update(&mut self, delta: f32) -> bool {
        self.timer -= delta;
        if self.timer <= 0.0 {
            self.timer = self.interval;
            true
        } else {
            false
        }
    }
}

/// Type d'entité à spawner
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnType {
    Asteroid,
    NpcHostile,
    NpcFriendly,
    Loot,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weapon_ready() {
        let mut weapon = Weapon::laser(0);
        assert!(weapon.is_ready());

        weapon.fire();
        assert!(!weapon.is_ready());

        weapon.update(0.6);
        assert!(weapon.is_ready());
    }

    #[test]
    fn test_propulsion_for_ship() {
        let propulsion = Propulsion::for_ship_class(ShipClass::Fighter);
        assert_eq!(propulsion.max_speed, 200.0);
        assert_eq!(propulsion.turn_rate, 3.0);

        let propulsion = Propulsion::for_ship_class(ShipClass::Cruiser);
        assert_eq!(propulsion.max_speed, 100.0);
        assert_eq!(propulsion.turn_rate, 1.0);
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
