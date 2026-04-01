//! R2 Server - Serveur de jeu MMO spatial
//!
//! # Architecture
//!
//! ```text
//! r2_server
//!   ├── main.rs          # Point d'entrée
//!   ├── auth.rs          # OAuth2 + JWT
//!   ├── network.rs       # WebSocket handler
//!   ├── ecs/             # Simulation Bevy headless
//!   │   ├── mod.rs
//!   │   ├── systems.rs
//!   │   └── resources.rs
//!   ├── spatial.rs       # Spatial hashing (AOI)
//!   └── db.rs            # PostgreSQL
//! ```

// #![warn(missing_docs)]  // TODO: Réactiver quand la doc sera complète
#![warn(clippy::all)]

use axum::{
    extract::{State, WebSocketUpgrade},
    routing::get,
    Router,
};
use futures_util::StreamExt;  // Pour .split()
use redis::aio::ConnectionManager;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod auth;
mod db;
mod error;
mod network;
mod spatial;

use error::Result;

/// État partagé de l'application
#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub redis: ConnectionManager,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialiser le logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("🚀 R2 Server starting...");

    // Charger les variables d'environnement
    dotenvy::dotenv().ok();

    // Connexion SQLite (dev local) - :memory: pour les tests, fichier pour persister
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite::memory:".to_string());

    info!("📦 Connecting to SQLite...");
    let db = SqlitePool::connect(&database_url).await?;
    info!("✅ SQLite connected");

    // Connexion Redis
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    info!("📦 Connecting to Redis...");
    let redis = ConnectionManager::new(redis::Client::open(redis_url.clone())?).await?;
    info!("✅ Redis connected");

    // Créer l'état partagé
    let state = AppState { db, redis };

    // Build le router
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        // WebSocket connection
        .route("/ws", get(ws_connect))
        .with_state(state);

    // Démarrer le serveur
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("🎮 R2 Server listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// WebSocket connection handler
async fn ws_connect(
    State(_state): State<AppState>,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(|socket| async move {
        // TODO: Implémenter la logique WebSocket
        // - Auth via JWT
        // - Join game server
        // - Message loop
        info!("🔌 New WebSocket connection");
        let (mut _sender, mut _receiver) = socket.split();

        // Pour l'instant, juste garder la connexion ouverte
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        info!("🔌 WebSocket connection closed");
    })
}
