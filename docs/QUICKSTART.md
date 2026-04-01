# Quick Start - R2 Dev Setup

> **Pour les développeurs qui veulent coder immédiatement**

---

## 🚀 Setup en 5 minutes

### Prérequis

```bash
# Rust (latest stable)
rustup update

# Node.js (pour Dioxus devtools)
node --version

# PostgreSQL 16+
psql --version

# Docker (optionnel, pour BDD locale)
docker --version
```

### Initialisation

```bash
# 1. Cloner
git clone https://github.com/yourusername/R2.git
cd R2

# 2. Créer le workspace (structure vide pour l'instant)
mkdir -p r2_core/src r2_server/src r2_client/src

# 3. Copier les Cargo.toml (voir ci-dessous)
```

### Workspace Root (`Cargo.toml`)

```toml
[workspace]
members = ["r2_core", "r2_server", "r2_client"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"

[workspace.dependencies]
bevy = "0.15"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres"] }
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"
```

### Core (`r2_core/Cargo.toml`)

```toml
[package]
name = "r2_core"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
bevy.workspace = true
```

### Core (`r2_core/src/lib.rs`)

```rust
use serde::{Deserialize, Serialize};

// Types de base partagés
pub type EntityId = u64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum R2Message {
    MoveTo { target: Vec2 },
    Ping { timestamp: u64 },
    Pong { timestamp: u64 },
}
```

### Server (`r2_server/Cargo.toml`)

```toml
[package]
name = "r2_server"
version.workspace = true
edition.workspace = true

[dependencies]
r2_core = { path = "../r2_core" }
axum.workspace = true
tokio.workspace = true
sqlx.workspace = true
serde_json.workspace = true
tower-http = { version = "0.5", features = ["cors"] }
```

### Server (`r2_server/src/main.rs`)

```rust
use axum::{Router, routing::get, Json};
use serde_json::json;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health_check));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("🚀 R2 Server listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "version": "0.1.0"
    }))
}
```

### Client (`r2_client/Cargo.toml`)

```toml
[package]
name = "r2_client"
version.workspace = true
edition.workspace = true

[dependencies]
r2_core = { path = "../r2_core" }
bevy.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]
```

### Client (`r2_client/src/lib.rs`)

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "R2 Client".to_string(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, movement)
        .run();
}

fn setup(mut commands: Commands) {
    // Caméra 2D
    commands.spawn(Camera2dBundle::default());

    // Player (cube rouge)
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::RED,
            custom_size: Some(Vec2::new(50., 50.)),
            ..default()
        },
        ..default()
    });
}

fn movement(
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Sprite>>,
) {
    let mut transform = query.single_mut();
    let speed = 200.;

    if keyboard.pressed(KeyCode::Z) || keyboard.pressed(KeyCode::W) {
        transform.translation.y += speed * time.delta_seconds();
    }
    if keyboard.pressed(KeyCode::S) {
        transform.translation.y -= speed * time.delta_seconds();
    }
    if keyboard.pressed(KeyCode::Q) || keyboard.pressed(KeyCode::A) {
        transform.translation.x -= speed * time.delta_seconds();
    }
    if keyboard.pressed(KeyCode::D) {
        transform.translation.x += speed * time.delta_seconds();
    }
}
```

---

## 🏃 Run

```bash
# Serveur
cd r2_server
cargo run

# Client (native)
cd r2_client
cargo run

# Client (web)
cargo install trunk
cd r2_client
trunk serve --open
```

---

## 📝 Prochaine étape

Voir [`01-architecture.md`](./01-architecture.md) pour la suite.

---

*Quick Start v0.1.0*
