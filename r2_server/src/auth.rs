//! Module d'authentification
//!
//! Gère OAuth2 et JWT.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

/// Claims JWT
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    /// Subject (player_id)
    pub sub: i64,
    /// Expiration
    pub exp: i64,
    /// Issued at
    pub iat: i64,
    /// Token type
    pub token_type: TokenType,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum TokenType {
    Access,
    Refresh,
}

/// Clé secrète JWT (TODO: depuis env)
const JWT_SECRET: &[u8] = b"your-256-bit-secret"; // À changer en prod!

/// Durée de vie des tokens (secondes)
const ACCESS_TOKEN_EXPIRATION: i64 = 15 * 60; // 15 minutes
const REFRESH_TOKEN_EXPIRATION: i64 = 30 * 24 * 60 * 60; // 30 jours

/// Erreur JWT
#[derive(Debug)]
pub enum JwtError {
    InvalidToken(String),
    Expired,
    Other(String),
}

impl std::fmt::Display for JwtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            Self::Expired => write!(f, "Token expired"),
            Self::Other(msg) => write!(f, "JWT error: {}", msg),
        }
    }
}

impl std::error::Error for JwtError {}

/// Générer un access token et un refresh token
pub fn generate_tokens(player_id: i64) -> Result<(String, String), JwtError> {
    let now = Utc::now();

    // Access Token
    let access_claims = Claims {
        sub: player_id,
        exp: (now + Duration::seconds(ACCESS_TOKEN_EXPIRATION)).timestamp(),
        iat: now.timestamp(),
        token_type: TokenType::Access,
    };

    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .map_err(|e| JwtError::Other(e.to_string()))?;

    // Refresh Token
    let refresh_claims = Claims {
        sub: player_id,
        exp: (now + Duration::seconds(REFRESH_TOKEN_EXPIRATION)).timestamp(),
        iat: now.timestamp(),
        token_type: TokenType::Refresh,
    };

    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .map_err(|e| JwtError::Other(e.to_string()))?;

    Ok((access_token, refresh_token))
}

/// Valider un token
pub fn validate_token(token: &str) -> Result<Claims, JwtError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::Expired,
        _ => JwtError::InvalidToken(e.to_string()),
    })?;

    Ok(token_data.claims)
}

/// Valider un access token
pub fn validate_access_token(token: &str) -> Result<i64, JwtError> {
    let claims = validate_token(token)?;

    if claims.token_type != TokenType::Access {
        return Err(JwtError::InvalidToken("Not an access token".to_string()));
    }

    Ok(claims.sub)
}

/// Valider un refresh token
pub fn validate_refresh_token(token: &str) -> Result<i64, JwtError> {
    let claims = validate_token(token)?;

    if claims.token_type != TokenType::Refresh {
        return Err(JwtError::InvalidToken("Not a refresh token".to_string()));
    }

    Ok(claims.sub)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_validate() {
        let player_id = 12345;

        let (access, refresh) = generate_tokens(player_id).unwrap();

        // Valider access token
        let access_player = validate_access_token(&access).unwrap();
        assert_eq!(access_player, player_id);

        // Valider refresh token
        let refresh_player = validate_refresh_token(&refresh).unwrap();
        assert_eq!(refresh_player, player_id);
    }

    #[test]
    fn test_invalid_token_type() {
        let player_id = 12345;
        let (access, _refresh) = generate_tokens(player_id).unwrap();

        // Essayer de valider un access token comme refresh token
        assert!(validate_refresh_token(&access).is_err());

        // Essayer de valider un refresh token comme access token
        assert!(validate_access_token(&_refresh).is_err());
    }
}
