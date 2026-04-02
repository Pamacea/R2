//! R2 Client - Client de jeu MMO spatial
//!
//! # Architecture
//!
//! ```text
//! r2_client
//!   ├── lib.rs           # Point d'entrée bibliothèque
//!   ├── main.rs          # Point d'entrée binaire (desktop)
//!   ├── network.rs       # WebSocket client
//!   ├── game.rs          # Logique de jeu + Bevy systems
//!   ├── renderer.rs      # Rendu des vaisseaux (triangles)
//!   ├── input.rs         # Gestion des entrées (ZQSD / flèches)
//!   └── interpolation.rs # Interpolation réseau
//! ```
//!
//! # Caractéristiques
//!
//! - **Rendu**: Vaisseaux comme triangles avec Bevy 2D
//! - **Input**: Support ZQSD (AZERTY) et flèches directionnelles
//! - **Réseau**: WebSocket avec interpolation pour mouvement fluide
//! - **Auth**: Token JWT supporté

#![cfg_attr(not(debug_assertions), warn(missing_docs))]
#![warn(clippy::all)]

pub mod game;
pub mod input;
pub mod interpolation;
pub mod network;
pub mod renderer;

use bevy::prelude::*;

/// Plugins R2 - Plugin principal du client
pub struct R2ClientPlugin;

impl Plugin for R2ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(game::GamePlugin);
    }
}

// Réexports pour faciliter l'utilisation
pub use game::{GamePlugin, spawn_enemy_ship};
pub use input::{InputPlugin, InputState};
pub use interpolation::{InterpolationPlugin, InterpolationState, InterpolationConfig};
pub use network::{R2WebSocketClient, ClientConfig, ClientError, ClientResult};
pub use renderer::{ShipRendererPlugin, ShipAssets, ShipRender};

/// Version du client
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Information sur le build
pub const BUILD_INFO: &str = "R2 Client";
