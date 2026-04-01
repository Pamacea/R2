//! Module de base de données
//!
//! Gère toutes les interactions avec PostgreSQL.

use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

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
    pool: PgPool,
}

impl PlayerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Trouver un joueur par OAuth provider + ID
    pub async fn find_by_oauth(
        &self,
        provider: &str,
        oauth_id: &str,
    ) -> Result<Option<Player>> {
        let player = sqlx::query(
            r#"
            SELECT player_id, username, email, oauth_provider, oauth_id,
                   created_at, last_login, is_banned
            FROM players
            WHERE oauth_provider = $1 AND oauth_id = $2
            "#,
        )
        .bind(provider)
        .bind(oauth_id)
        .fetch_optional(&self.pool)
        .await?
        .map(|row| Player {
            player_id: row.get("player_id"),
            username: row.get("username"),
            email: row.get("email"),
            oauth_provider: row.get("oauth_provider"),
            oauth_id: row.get("oauth_id"),
            created_at: row.get("created_at"),
            last_login: row.get("last_login"),
            is_banned: row.get("is_banned"),
        });

        Ok(player)
    }

    /// Créer un nouveau joueur
    pub async fn create(
        &self,
        username: &str,
        email: Option<&str>,
        provider: &str,
        oauth_id: &str,
    ) -> Result<Player> {
        let player = sqlx::query(
            r#"
            INSERT INTO players (username, email, oauth_provider, oauth_id, last_login)
            VALUES ($1, $2, $3, $4, NOW())
            RETURNING player_id, username, email, oauth_provider, oauth_id,
                      created_at, last_login, is_banned
            "#,
        )
        .bind(username)
        .bind(email)
        .bind(provider)
        .bind(oauth_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Player {
            player_id: player.get("player_id"),
            username: player.get("username"),
            email: player.get("email"),
            oauth_provider: player.get("oauth_provider"),
            oauth_id: player.get("oauth_id"),
            created_at: player.get("created_at"),
            last_login: player.get("last_login"),
            is_banned: player.get("is_banned"),
        })
    }

    /// Créer ou mettre à jour un joueur via OAuth (upsert)
    pub async fn upsert_by_oauth(
        &self,
        username: &str,
        email: Option<&str>,
        provider: &str,
        oauth_id: &str,
    ) -> Result<Player> {
        let player = sqlx::query(
            r#"
            INSERT INTO players (username, email, oauth_provider, oauth_id, last_login)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (oauth_provider, oauth_id)
            DO UPDATE SET last_login = NOW(), username = $1
            RETURNING player_id, username, email, oauth_provider, oauth_id,
                      created_at, last_login, is_banned
            "#,
        )
        .bind(username)
        .bind(email)
        .bind(provider)
        .bind(oauth_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Player {
            player_id: player.get("player_id"),
            username: player.get("username"),
            email: player.get("email"),
            oauth_provider: player.get("oauth_provider"),
            oauth_id: player.get("oauth_id"),
            created_at: player.get("created_at"),
            last_login: player.get("last_login"),
            is_banned: player.get("is_banned"),
        })
    }
}

/// Repository pour les personnages
pub struct CharacterRepository {
    pool: PgPool,
}

impl CharacterRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Lister les personnages d'un joueur
    pub async fn list_by_player(&self, player_id: i64) -> Result<Vec<Character>> {
        let rows = sqlx::query(
            r#"
            SELECT character_id, player_id, name, level, xp,
                   zone_id, x, y, z, rotation, ship_class, credits
            FROM characters
            WHERE player_id = $1 AND is_deleted = FALSE
            ORDER BY created_at
            "#,
        )
        .bind(player_id)
        .fetch_all(&self.pool)
        .await?;

        let characters = rows
            .iter()
            .map(|row| Character {
                character_id: row.get("character_id"),
                player_id: row.get("player_id"),
                name: row.get("name"),
                level: row.get("level"),
                xp: row.get("xp"),
                zone_id: row.get("zone_id"),
                x: row.get("x"),
                y: row.get("y"),
                z: row.get("z"),
                rotation: row.get("rotation"),
                ship_class: row.get("ship_class"),
                credits: row.get("credits"),
            })
            .collect();

        Ok(characters)
    }

    /// Créer un nouveau personnage
    pub async fn create(
        &self,
        player_id: i64,
        name: &str,
        ship_class: &str,
    ) -> Result<Character> {
        let row = sqlx::query(
            r#"
            INSERT INTO characters (player_id, name, ship_class, zone_id, x, y, z, rotation)
            VALUES ($1, $2, $3, 1, 0.0, 0.0, 0.0, 0.0)
            RETURNING character_id, player_id, name, level, xp,
                      zone_id, x, y, z, rotation, ship_class, credits
            "#,
        )
        .bind(player_id)
        .bind(name)
        .bind(ship_class)
        .fetch_one(&self.pool)
        .await?;

        Ok(Character {
            character_id: row.get("character_id"),
            player_id: row.get("player_id"),
            name: row.get("name"),
            level: row.get("level"),
            xp: row.get("xp"),
            zone_id: row.get("zone_id"),
            x: row.get("x"),
            y: row.get("y"),
            z: row.get("z"),
            rotation: row.get("rotation"),
            ship_class: row.get("ship_class"),
            credits: row.get("credits"),
        })
    }

    /// Sauvegarder la position d'un personnage
    pub async fn save_position(
        &self,
        character_id: i64,
        zone_id: i16,
        x: f32,
        y: f32,
        z: f32,
        rotation: f32,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE characters
            SET zone_id = $1, x = $2, y = $3, z = $4, rotation = $5, updated_at = NOW()
            WHERE character_id = $6
            "#,
        )
        .bind(zone_id)
        .bind(x)
        .bind(y)
        .bind(z)
        .bind(rotation)
        .bind(character_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
