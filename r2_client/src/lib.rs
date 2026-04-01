//! R2 Client - Client de jeu MMO spatial
//!
//! # Architecture
//!
//! ```text
//! r2_client
//!   ├── lib.rs           # Point d'entrée bibliothèque
//!   ├── main.rs          # Point d'entrée binaire
//!   ├── network.rs       # WebSocket client
//!   ├── game.rs          # Logique de jeu
//!   └── ui.rs            # Interface Dioxus
//! ```

#![cfg_attr(not(debug_assertions), warn(missing_docs))]
#![warn(clippy::all)]

pub mod game;
pub mod network;

use bevy::prelude::*;

/// Plugins R2
pub struct R2ClientPlugin;

impl Plugin for R2ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(game::GamePlugin);
        // TODO: Ajouter UI plugin quand Dioxus sera intégré
    }
}
