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
    extract::{Path, State, WebSocketUpgrade},
    http::Method,
    routing::{get, post},
    Json, Router,
};
use tower_http::cors::{Any, CorsLayer};
use redis::aio::ConnectionManager;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

mod auth;
mod db;
mod ecs;
mod error;
mod game;
mod network;
mod spatial;

use auth::{AuthState, OAuthProvider};
use game::GameState;

/// État partagé de l'application
#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub redis: ConnectionManager,
    pub auth: AuthState,
    pub game: GameState,
}

/// Login request
#[derive(serde::Deserialize)]
struct LoginRequest {
    provider: String,
    code: String,
}

/// Login response
#[derive(serde::Serialize)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    player_id: i64,
}

/// Refresh request
#[derive(serde::Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

#[tokio::main]
async fn main() -> error::Result<()> {
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

    // Initialiser l'auth
    let auth = AuthState::new();

    // Initialiser le game state
    let game = GameState::new(db.clone());

    // Créer l'état partagé
    let state = AppState { db, redis, auth, game: game.clone() };

    // Lancer le game loop en arrière-plan
    tokio::spawn(async move {
        game::run_game_loop(game).await;
    });

    // Configurer CORS (dev: allow any origin)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .expose_headers(Any);

    // Build le router - IMPORTANT: layer cors APRES les routes pour qu'il soit executé AVANT
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        // OAuth endpoints
        .route("/auth/url/:provider", get(get_auth_url))
        .route("/auth/login", post(oauth_login))
        .route("/auth/refresh", post(refresh_token))
        // WebSocket connection
        .route("/ws", get(ws_connect))
        .with_state(state)
        .layer(cors);

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

/// Obtenir l'URL d'autorisation OAuth
async fn get_auth_url(
    Path(provider): Path<String>,
    State(state): State<AppState>,
) -> std::result::Result<Json<auth::AuthResponse>, error::AppError> {
    let provider = provider.parse::<OAuthProvider>().map_err(|e| {
        error::AppError::Auth(format!("Invalid provider: {}", e))
    })?;

    let response = auth::get_authorization_url(provider, &state.auth.config)
        .await
        .map_err(|e| error::AppError::Auth(e.to_string()))?;

    Ok(Json(response))
}

/// Login via OAuth (echange le code contre des tokens JWT)
async fn oauth_login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> std::result::Result<Json<LoginResponse>, error::AppError> {
    let provider = req.provider.parse::<OAuthProvider>().map_err(|e| {
        error::AppError::Auth(format!("Invalid provider: {}", e))
    })?;

    // Echanger le code contre les infos utilisateur
    let user_info = auth::exchange_code(provider, &req.code, &state.auth.config)
        .await
        .map_err(|e| error::AppError::Auth(e.to_string()))?;

    // TODO: Creer ou mettre a jour le joueur en base de donnees
    // Pour l'instant, on utilise un player_id base sur un hash de l'ID OAuth
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    format!("{}:{}", user_info.provider as i8, user_info.id).hash(&mut hasher);
    let player_id = hasher.finish() as i64;

    info!(
        "User logged in: {} via {:?}, player_id={}",
        user_info.username, user_info.provider, player_id
    );

    // Generer les tokens JWT
    let (access_token, refresh_token) = auth::generate_tokens(player_id)
        .map_err(|e| error::AppError::Auth(format!("Failed to generate tokens: {}", e)))?;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        expires_in: 15 * 60, // 15 minutes
        player_id,
    }))
}

/// Rafraichir un token d'acces
async fn refresh_token(
    Json(req): Json<RefreshRequest>,
) -> std::result::Result<Json<LoginResponse>, error::AppError> {
    // Valider le refresh token
    let player_id = auth::validate_refresh_token(&req.refresh_token)
        .map_err(|e| error::AppError::Auth(format!("Invalid refresh token: {}", e)))?;

    // Generer nouveaux tokens
    let (access_token, refresh_token) = auth::generate_tokens(player_id)
        .map_err(|e| error::AppError::Auth(format!("Failed to generate tokens: {}", e)))?;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        expires_in: 15 * 60,
        player_id,
    }))
}

/// WebSocket connection handler
async fn ws_connect(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(|socket| async move {
        if let Err(e) = network::handle_websocket(socket, state).await {
            error!("WebSocket handler error: {:?}", e);
        }
    })
}
