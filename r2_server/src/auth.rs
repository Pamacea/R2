//! Module d'authentification
//!
//! Gère OAuth2 (Discord, GitHub), Email/Password et JWT.

#![allow(dead_code)]  // TODO: À utiliser dans les prochaines versions

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use base64::Engine;

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Configuration OAuth2
#[derive(Clone)]
pub struct OAuthConfig {
    pub github_client_id: Option<String>,
    pub github_client_secret: Option<String>,
    pub discord_client_id: Option<String>,
    pub discord_client_secret: Option<String>,
    pub base_url: String,
}

impl OAuthConfig {
    /// Charger depuis les variables d'environnement
    pub fn from_env() -> Self {
        let base_url = std::env::var("OAUTH_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:3001".to_string());

        let github_client_id = std::env::var("GITHUB_CLIENT_ID").ok();
        let github_client_secret = std::env::var("GITHUB_CLIENT_SECRET").ok();
        let discord_client_id = std::env::var("DISCORD_CLIENT_ID").ok();
        let discord_client_secret = std::env::var("DISCORD_CLIENT_SECRET").ok();

        if github_client_id.is_some() || github_client_secret.is_some() {
            tracing::info!("GitHub OAuth configured");
        } else {
            tracing::warn!("GitHub OAuth not configured (missing GITHUB_CLIENT_ID or GITHUB_CLIENT_SECRET)");
        }

        if discord_client_id.is_some() || discord_client_secret.is_some() {
            tracing::info!("Discord OAuth configured");
        } else {
            tracing::warn!("Discord OAuth not configured (missing DISCORD_CLIENT_ID or DISCORD_CLIENT_SECRET)");
        }

        Self {
            github_client_id,
            github_client_secret,
            discord_client_id,
            discord_client_secret,
            base_url,
        }
    }
}

/// État partagé pour l'authentification
#[derive(Clone)]
pub struct AuthState {
    pub config: Arc<OAuthConfig>,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            config: Arc::new(OAuthConfig::from_env()),
        }
    }
}

impl Default for AuthState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// JWT
// ============================================================================

/// Claims JWT
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TokenType {
    Access,
    Refresh,
}

/// Obtenir la clé secrète JWT depuis les variables d'environnement
fn jwt_secret() -> Vec<u8> {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev-secret-change-in-production-256-bits".to_string())
        .into_bytes()
}

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
    let secret = jwt_secret();

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
        &EncodingKey::from_secret(&secret),
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
        &EncodingKey::from_secret(&secret),
    )
    .map_err(|e| JwtError::Other(e.to_string()))?;

    Ok((access_token, refresh_token))
}

/// Valider un token
pub fn validate_token(token: &str) -> Result<Claims, JwtError> {
    let secret = jwt_secret();

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&secret),
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

// ============================================================================
// OAUTH2 HANDLERS
// ============================================================================

/// Provider OAuth2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OAuthProvider {
    GitHub,
    Discord,
}

impl std::str::FromStr for OAuthProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "github" => Ok(Self::GitHub),
            "discord" => Ok(Self::Discord),
            _ => Err(format!("Unknown provider: {}", s)),
        }
    }
}

/// Réponse d'autorisation OAuth
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub auth_url: String,
    pub csrf_token: String,
}

/// Informations utilisateur OAuth (normalisées)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub provider: OAuthProvider,
    pub id: String,
    pub email: String,
    pub username: String,
    pub avatar_url: Option<String>,
}

/// Erreur OAuth
#[derive(Debug)]
pub enum OAuthError {
    ProviderNotConfigured(String),
    InvalidProvider(String),
    ExchangeFailed(String),
    UserInfoFailed(String),
}

impl std::fmt::Display for OAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProviderNotConfigured(p) => write!(f, "Provider not configured: {}", p),
            Self::InvalidProvider(p) => write!(f, "Invalid provider: {}", p),
            Self::ExchangeFailed(msg) => write!(f, "Token exchange failed: {}", msg),
            Self::UserInfoFailed(msg) => write!(f, "Failed to fetch user info: {}", msg),
        }
    }
}

impl std::error::Error for OAuthError {}

/// Générer l'URL d'autorisation OAuth
pub async fn get_authorization_url(
    provider: OAuthProvider,
    config: &OAuthConfig,
) -> Result<AuthResponse, OAuthError> {
    use uuid::Uuid;

    let csrf_token = Uuid::new_v4().to_string();

    let (auth_url, _client_id) = match provider {
        OAuthProvider::GitHub => {
            let client_id = config
                .github_client_id
                .as_ref()
                .ok_or_else(|| OAuthError::ProviderNotConfigured("GitHub".to_string()))?;

            let url = format!(
                "https://github.com/login/oauth/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}",
                client_id,
                urlencoding::encode(&format!("{}/auth/callback", config.base_url)),
                urlencoding::encode("user:email read:user"),
                urlencoding::encode(&csrf_token)
            );

            (url, client_id.clone())
        }
        OAuthProvider::Discord => {
            let client_id = config
                .discord_client_id
                .as_ref()
                .ok_or_else(|| OAuthError::ProviderNotConfigured("Discord".to_string()))?;

            let url = format!(
                "https://discord.com/oauth2/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}",
                client_id,
                urlencoding::encode(&format!("{}/auth/callback", config.base_url)),
                urlencoding::encode("identify email"),
                urlencoding::encode(&csrf_token)
            );

            (url, client_id.clone())
        }
    };

    tracing::info!("Generated OAuth URL for {:?}", provider);

    Ok(AuthResponse { auth_url, csrf_token })
}

/// Échanger le code contre des tokens et récupérer les infos utilisateur
pub async fn exchange_code(
    provider: OAuthProvider,
    code: &str,
    config: &OAuthConfig,
) -> Result<OAuthUserInfo, OAuthError> {
    let user_info = match provider {
        OAuthProvider::GitHub => {
            let client_id = config
                .github_client_id
                .as_ref()
                .ok_or_else(|| OAuthError::ProviderNotConfigured("GitHub".to_string()))?;
            let client_secret = config
                .github_client_secret
                .as_ref()
                .ok_or_else(|| OAuthError::ProviderNotConfigured("GitHub".to_string()))?;

            exchange_github_code(client_id, client_secret, code, &config.base_url).await?
        }
        OAuthProvider::Discord => {
            let client_id = config
                .discord_client_id
                .as_ref()
                .ok_or_else(|| OAuthError::ProviderNotConfigured("Discord".to_string()))?;
            let client_secret = config
                .discord_client_secret
                .as_ref()
                .ok_or_else(|| OAuthError::ProviderNotConfigured("Discord".to_string()))?;

            exchange_discord_code(client_id, client_secret, code, &config.base_url).await?
        }
    };

    Ok(user_info)
}

/// Échanger le code GitHub contre des tokens
async fn exchange_github_code(
    client_id: &str,
    client_secret: &str,
    code: &str,
    base_url: &str,
) -> Result<OAuthUserInfo, OAuthError> {
    let http_client = reqwest::Client::new();

    // GitHub utilise Basic Auth avec client_id:client_secret
    let auth_header = format!("Basic {}", base64::engine::general_purpose::STANDARD.encode(&format!("{}:{}", client_id, client_secret)));

    // Échanger le code contre des tokens
    let token_response: GitHubTokenResponse = http_client
        .post("https://github.com/login/oauth/access_token")
        .header("Authorization", auth_header)
        .form(&[
            ("code", code),
            ("redirect_uri", &format!("{}/auth/callback", base_url)),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| OAuthError::ExchangeFailed(e.to_string()))?
        .json()
        .await
        .map_err(|e| OAuthError::ExchangeFailed(e.to_string()))?;

    // Récupérer les infos utilisateur
    fetch_github_user_info(&token_response.access_token).await
}

/// Échanger le code Discord contre des tokens
async fn exchange_discord_code(
    client_id: &str,
    client_secret: &str,
    code: &str,
    base_url: &str,
) -> Result<OAuthUserInfo, OAuthError> {
    let http_client = reqwest::Client::new();

    // Échanger le code contre des tokens
    let token_response: DiscordTokenResponse = http_client
        .post("https://discord.com/api/oauth2/token")
        .form(&[
            ("code", code),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("redirect_uri", &format!("{}/auth/callback", base_url)),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| OAuthError::ExchangeFailed(e.to_string()))?
        .json()
        .await
        .map_err(|e| OAuthError::ExchangeFailed(e.to_string()))?;

    // Récupérer les infos utilisateur
    fetch_discord_user_info(&token_response.access_token).await
}

#[derive(Deserialize)]
struct GitHubTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct DiscordTokenResponse {
    access_token: String,
}

/// Récupérer les infos utilisateur GitHub
async fn fetch_github_user_info(access_token: &str) -> Result<OAuthUserInfo, OAuthError> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "R2-MMO")
        .send()
        .await
        .map_err(|e| OAuthError::UserInfoFailed(e.to_string()))?;

    #[derive(Debug, Deserialize)]
    struct GitHubUserInfo {
        id: u64,
        email: Option<String>,
        login: String,
        avatar_url: String,
    }

    let info: GitHubUserInfo = response
        .json()
        .await
        .map_err(|e| OAuthError::UserInfoFailed(e.to_string()))?;

    // Récupérer l'email principal si pas fourni
    let email = if info.email.is_some() {
        info.email.unwrap()
    } else {
        // Appel supplémentaire pour les emails
        let emails_response = client
            .get("https://api.github.com/user/emails")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "R2-MMO")
            .send()
            .await
            .map_err(|e| OAuthError::UserInfoFailed(e.to_string()))?;

        #[derive(Debug, Deserialize)]
        struct GitHubEmail {
            email: String,
            primary: bool,
        }

        let emails: Vec<GitHubEmail> = emails_response
            .json()
            .await
            .map_err(|e| OAuthError::UserInfoFailed(e.to_string()))?;

        emails
            .into_iter()
            .find(|e| e.primary)
            .map(|e| e.email)
            .unwrap_or_else(|| format!("{}@users.noreply.github.com", info.login))
    };

    Ok(OAuthUserInfo {
        provider: OAuthProvider::GitHub,
        id: info.id.to_string(),
        email,
        username: info.login,
        avatar_url: Some(info.avatar_url),
    })
}

/// Récupérer les infos utilisateur Discord
async fn fetch_discord_user_info(access_token: &str) -> Result<OAuthUserInfo, OAuthError> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://discord.com/api/users/@me")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| OAuthError::UserInfoFailed(e.to_string()))?;

    #[derive(Debug, Deserialize)]
    struct DiscordUserInfo {
        id: String,
        email: String,
        username: String,
        global_name: Option<String>,
        avatar: Option<String>,
    }

    let info: DiscordUserInfo = response
        .json()
        .await
        .map_err(|e| OAuthError::UserInfoFailed(e.to_string()))?;

    let avatar_url = info.avatar.map(|avatar| {
        format!(
            "https://cdn.discordapp.com/avatars/{}/{}.png",
            info.id, avatar
        )
    });

    Ok(OAuthUserInfo {
        provider: OAuthProvider::Discord,
        id: info.id,
        email: info.email,
        username: info.global_name.unwrap_or(info.username),
        avatar_url,
    })
}

// ============================================================================
// WEBSOCKET AUTH
// ============================================================================

/// Message d'authentification WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsAuthMessage {
    pub token: String,
}

/// Message de réponse d'authentification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsAuthResponse {
    pub success: bool,
    pub player_id: Option<i64>,
    pub error: Option<String>,
}

/// Valider le token pour une connexion WebSocket
pub fn validate_ws_token(token: &str) -> Result<i64, WsAuthError> {
    validate_access_token(token).map_err(|e| match e {
        JwtError::Expired => WsAuthError::TokenExpired,
        JwtError::InvalidToken(msg) => WsAuthError::InvalidToken(msg),
        JwtError::Other(msg) => WsAuthError::Other(msg),
    })
}

/// Erreur d'authentification WebSocket
#[derive(Debug)]
pub enum WsAuthError {
    MissingToken,
    InvalidToken(String),
    TokenExpired,
    Other(String),
}

impl std::fmt::Display for WsAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingToken => write!(f, "Missing authentication token"),
            Self::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            Self::TokenExpired => write!(f, "Token expired"),
            Self::Other(msg) => write!(f, "Authentication error: {}", msg),
        }
    }
}

impl std::error::Error for WsAuthError {}

// ============================================================================
// TESTS
// ============================================================================

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

    #[test]
    fn test_oauth_provider_from_str() {
        assert_eq!("github".parse::<OAuthProvider>().unwrap(), OAuthProvider::GitHub);
        assert_eq!("discord".parse::<OAuthProvider>().unwrap(), OAuthProvider::Discord);
        assert_eq!("GITHUB".parse::<OAuthProvider>().unwrap(), OAuthProvider::GitHub);
        assert!("unknown".parse::<OAuthProvider>().is_err());
    }
}
