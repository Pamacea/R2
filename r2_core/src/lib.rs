//! R2 Core - Types et messages partagés entre client et serveur
//!
//! Ce crate contient toute la logique commune :
//! - Composants ECS (Position, Stats, etc.)
//! - Messages réseau (R2Message)
//! - Types de domaine (EntityId, Faction, etc.)
//!
//! # Architecture
//!
//! ```text
//! r2_core (no std dependency)
//!   ├── components/    // Composants ECS partagés
//!   ├── network/       // Protocole de messages
//!   └── types/         // Types de domaine
//! ```

// #![warn(missing_docs)]  // TODO: Réactiver quand la doc sera complète
#![warn(clippy::all)]

pub mod components;
pub mod network;
pub mod types;

// Réexports pour faciliter l'utilisation
pub use components::*;
pub use network::*;
pub use types::*;
