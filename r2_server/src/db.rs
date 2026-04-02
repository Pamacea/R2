//! Module de base de données
//!
//! Gère toutes les interactions avec SQLite (dev local) / PostgreSQL (prod).

#![allow(dead_code)]  // TODO: À utiliser dans les prochaines versions

use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Joueur (compte utilisateur)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Player {
    pub player_id: i64,
    pub username: String,
    pub email: Option<String>,
    pub oauth_provider: Option<String>,
    pub oauth_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    /// Stocké comme INTEGER en SQLite (0 = false, 1 = true)
    pub is_banned: bool,
}

/// Personnage
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
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
        provider: &str,
        oauth_id: &str,
    ) -> Result<Option<Player>> {
        let player = sqlx::query_as::<_, Player>(
            r#"
            SELECT
                player_id, username, email, oauth_provider, oauth_id,
                created_at, last_login, is_banned
            FROM players
            WHERE oauth_provider = ? AND oauth_id = ?
            "#,
        )
        .bind(provider)
        .bind(oauth_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(player)
    }

    /// Trouver un joueur par ID
    pub async fn find_by_id(&self, player_id: i64) -> Result<Option<Player>> {
        let player = sqlx::query_as::<_, Player>(
            r#"
            SELECT
                player_id, username, email, oauth_provider, oauth_id,
                created_at, last_login, is_banned
            FROM players
            WHERE player_id = ?
            "#,
        )
        .bind(player_id)
        .fetch_optional(&self.pool)
        .await?;

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
        let now = Utc::now();
        let created_at_str = now.to_rfc3339();

        let player_id = sqlx::query(
            r#"
            INSERT INTO players (username, email, oauth_provider, oauth_id, created_at, last_login)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(username)
        .bind(email)
        .bind(provider)
        .bind(oauth_id)
        .bind(&created_at_str)
        .bind(&created_at_str)
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        self.find_by_id(player_id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to retrieve created player".to_string()))
    }

    /// Mettre à jour la date de dernière connexion
    pub async fn update_last_login(&self, player_id: i64) -> Result<()> {
        let now = Utc::now();
        let last_login_str = now.to_rfc3339();

        sqlx::query(
            r#"
            UPDATE players
            SET last_login = ?
            WHERE player_id = ?
            "#,
        )
        .bind(&last_login_str)
        .bind(player_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Créer ou mettre à jour un joueur via OAuth (upsert)
    pub async fn upsert_by_oauth(
        &self,
        username: &str,
        email: Option<&str>,
        provider: &str,
        oauth_id: &str,
    ) -> Result<Player> {
        // D'abord, essayer de trouver un joueur existant
        if let Some(mut existing_player) = self.find_by_oauth(provider, oauth_id).await? {
            // Mise à jour du username/email si nécessaire
            let now = Utc::now();
            let last_login_str = now.to_rfc3339();

            sqlx::query(
                r#"
                UPDATE players
                SET username = ?, email = ?, last_login = ?
                WHERE player_id = ?
                "#,
            )
            .bind(username)
            .bind(email)
            .bind(&last_login_str)
            .bind(existing_player.player_id)
            .execute(&self.pool)
            .await?;

            existing_player.username = username.to_string();
            existing_player.email = email.map(|e| e.to_string());
            existing_player.last_login = Some(now);

            return Ok(existing_player);
        }

        // Si pas trouvé, créer un nouveau joueur
        self.create(username, email, provider, oauth_id).await
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
    pub async fn list_by_player(&self, player_id: i64) -> Result<Vec<Character>> {
        let characters = sqlx::query_as::<_, Character>(
            r#"
            SELECT character_id, player_id, name, level, xp, zone_id, x, y, z, rotation, ship_class, credits
            FROM characters
            WHERE player_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(player_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(characters)
    }

    /// Créer un nouveau personnage
    pub async fn create(
        &self,
        player_id: i64,
        name: &str,
        ship_class: &str,
    ) -> Result<Character> {
        let character_id = sqlx::query(
            r#"
            INSERT INTO characters (player_id, name, ship_class)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(player_id)
        .bind(name)
        .bind(ship_class)
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        // Récupérer le personnage créé
        let character = sqlx::query_as::<_, Character>(
            r#"
            SELECT character_id, player_id, name, level, xp, zone_id, x, y, z, rotation, ship_class, credits
            FROM characters
            WHERE character_id = ?
            "#,
        )
        .bind(character_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(character)
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
            SET zone_id = ?, x = ?, y = ?, z = ?, rotation = ?, updated_at = datetime('now')
            WHERE character_id = ?
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

    /// Supprimer un personnage
    pub async fn delete(&self, character_id: i64) -> Result<()> {
        let result = sqlx::query("DELETE FROM characters WHERE character_id = ?")
            .bind(character_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(crate::error::AppError::NotFound(format!(
                "Character {} not found",
                character_id
            )));
        }

        Ok(())
    }

    /// Ajouter de l'XP à un personnage (avec level up automatique)
    ///
    /// Formule de level up: XP requis = level * 100
    /// Exemple: level 1 -> 2 = 100 XP, level 2 -> 3 = 200 XP, etc.
    pub async fn add_xp(&self, character_id: i64, xp_amount: i64) -> Result<()> {
        if xp_amount <= 0 {
            return Err(crate::error::AppError::Internal(
                "XP amount must be positive".to_string(),
            ));
        }

        // Récupérer le personnage actuel
        let character: Character = sqlx::query_as::<_, Character>(
            r#"
            SELECT character_id, player_id, name, level, xp, zone_id, x, y, z, rotation, ship_class, credits
            FROM characters
            WHERE character_id = ?
            "#,
        )
        .bind(character_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!("Character {} not found", character_id))
        })?;

        // Calculer les nouveaux niveau et XP
        let mut new_level = character.level;
        let mut remaining_xp = character.xp + xp_amount;

        // XP requis pour chaque niveau atteint
        while remaining_xp >= (new_level as i64) * 100 {
            remaining_xp -= (new_level as i64) * 100;
            new_level += 1;
        }

        // Mettre à jour en base
        sqlx::query(
            r#"
            UPDATE characters
            SET level = ?, xp = ?, updated_at = datetime('now')
            WHERE character_id = ?
            "#,
        )
        .bind(new_level)
        .bind(remaining_xp)
        .bind(character_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Trouver un personnage par ID
    pub async fn find_by_id(&self, character_id: i64) -> Result<Option<Character>> {
        let character = sqlx::query_as::<_, Character>(
            r#"
            SELECT character_id, player_id, name, level, xp, zone_id, x, y, z, rotation, ship_class, credits
            FROM characters
            WHERE character_id = ?
            "#,
        )
        .bind(character_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(character)
    }
}
