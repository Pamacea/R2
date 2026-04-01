//! Module de base de données
//!
//! Gère toutes les interactions avec SQLite (dev local) / PostgreSQL (prod).

use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};

/// Joueur (compte utilisateur)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub player_id: i64,
    pub username: String,
    pub email: Option<String>,
    pub oauth_provider: Option<String>,
    pub oauth_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_banned: bool,
}

/// Personnage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub character_id: i64,
    pub player_id: i64,
    pub name: String,
    pub level: i16,
    pub xp: i64,
    pub zone_id: i16,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rotation: f32,
    pub ship_class: String,
    pub credits: i64,
}

/// Repository pour les joueurs
pub struct PlayerRepository {
    pool: SqlitePool,
}

impl PlayerRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Trouver un joueur par OAuth provider + ID
    pub async fn find_by_oauth(
        &self,
        _provider: &str,
        _oauth_id: &str,
    ) -> Result<Option<Player>> {
        // TODO: Implémenter avec SQLite
        Ok(None)
    }

    /// Créer un nouveau joueur
    pub async fn create(
        &self,
        _username: &str,
        _email: Option<&str>,
        _provider: &str,
        _oauth_id: &str,
    ) -> Result<Player> {
        // TODO: Implémenter avec SQLite
        Err(crate::error::AppError::Internal("Not implemented".to_string()))
    }

    /// Créer ou mettre à jour un joueur via OAuth (upsert)
    pub async fn upsert_by_oauth(
        &self,
        _username: &str,
        _email: Option<&str>,
        _provider: &str,
        _oauth_id: &str,
    ) -> Result<Player> {
        // TODO: Implémenter avec SQLite
        Err(crate::error::AppError::Internal("Not implemented".to_string()))
    }
}

/// Repository pour les personnages
pub struct CharacterRepository {
    pool: SqlitePool,
}

impl CharacterRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Lister les personnages d'un joueur
    pub async fn list_by_player(&self, _player_id: i64) -> Result<Vec<Character>> {
        // TODO: Implémenter avec SQLite
        Ok(vec![])
    }

    /// Créer un nouveau personnage
    pub async fn create(
        &self,
        _player_id: i64,
        _name: &str,
        _ship_class: &str,
    ) -> Result<Character> {
        // TODO: Implémenter avec SQLite
        Err(crate::error::AppError::Internal("Not implemented".to_string()))
    }

    /// Sauvegarder la position d'un personnage
    pub async fn save_position(
        &self,
        _character_id: i64,
        _zone_id: i16,
        _x: f32,
        _y: f32,
        _z: f32,
        _rotation: f32,
    ) -> Result<()> {
        // TODO: Implémenter avec SQLite
        Ok(())
    }
}
