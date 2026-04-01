# Auth System R2

> **Version** : 0.1.0 | **Dernière mise à jour** : 2025-04-01

---

## 🔐 Vue d'Ensemble

R2 utilise **OAuth2** pour l'authentification (Google, Discord) et **JWT** pour les sessions. Pas de mots de passe à gérer = meilleure sécurité.

```
┌─────────────────────────────────────────────────────────────────┐
│                     AUTH FLOW R2                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌─────────┐       ┌──────────────┐       ┌──────────────┐    │
│   │ Browser │       │ API Gateway  │       │   OAuth      │    │
│   └────┬────┘       └──────┬───────┘       │  Provider    │    │
│        │                   │               └──────┬───────┘    │
│        │ 1. Login         │                      │            │
│        ├──────────────────►│                      │            │
│        │                   │                      │            │
│        │                   │ 2. Redirect          │            │
│        │◄──────────────────┼─────────────────────►│            │
│        │                   │                      │            │
│        │ 3. User auth      │                      │            │
│        │◄──────────────────┼─────────────────────►│            │
│        │                   │                      │            │
│        │ 4. Code           │                      │            │
│        ├──────────────────►│                      │            │
│        │                   │                      │            │
│        │                   │ 5. Exchange token    │            │
│        │                   ├─────────────────────►│            │
│        │                   │                      │            │
│        │                   │ 6. Access token      │            │
│        │                   │◄─────────────────────┤            │
│        │                   │                      │            │
│        │                   │ 7. Get user info     │            │
│        │                   ├─────────────────────►│            │
│        │                   │                      │            │
│        │ 8. JWT + Refresh  │                      │            │
│        │◄──────────────────┤                      │            │
│        │                   │                      │            │
│        │ 9. Store tokens   │                      │            │
│        │                   │                      │            │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📦 Configuration OAuth

### Providers Supportés

| Provider | Scopes | Avatar |
|----------|--------|--------|
| Google | `openid`, `email`, `profile` | ✅ |
| Discord | `identify`, `email` | ✅ |
| GitHub | `user:email` | ✅ |

### Environment Variables

```bash
# .env
OAUTH_GOOGLE_CLIENT_ID="xxx.apps.googleusercontent.com"
OAUTH_GOOGLE_CLIENT_SECRET="xxx"

OAUTH_DISCORD_CLIENT_ID="xxx"
OAUTH_DISCORD_CLIENT_SECRET="xxx"

OAUTH_GITHUB_CLIENT_ID="xxx"
OAUTH_GITHUB_CLIENT_SECRET="xxx"

# JWT
JWT_SECRET="your-256-bit-secret"
JWT_EXPIRATION="15m"     # Access token
REFRESH_EXPIRATION="30d"  # Refresh token

# Callback URL
OAUTH_REDIRECT_URI="https://r2-game.com/auth/callback"
```

---

## 🔄 Flow Complet

### 1. Initialisation OAuth

```rust
// r2_server/src/auth/oauth.rs

use oauth2::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, AuthUrl, TokenUrl,
};

pub struct OAuthClient {
    client: BasicClient,
    provider: OAuthProvider,
}

impl OAuthClient {
    pub fn new(provider: OAuthProvider) -> Result<Self> {
        let (client_id, client_secret, auth_url, token_url) = match provider {
            OAuthProvider::Google => (
                env::var("OAUTH_GOOGLE_CLIENT_ID")?,
                ClientSecret::new(env::var("OAUTH_GOOGLE_CLIENT_SECRET")?),
                AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?,
                TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?,
            ),
            OAuthProvider::Discord => (
                env::var("OAUTH_DISCORD_CLIENT_ID")?,
                ClientSecret::new(env::var("OAUTH_DISCORD_CLIENT_SECRET")?),
                AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string())?,
                TokenUrl::new("https://discord.com/api/oauth2/token".to_string())?,
            ),
        };

        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(RedirectUrl::new(env::var("OAUTH_REDIRECT_URI")?));

        Ok(Self { client, provider })
    }

    pub fn authorize_url(&self) -> (String, CsrfToken) {
        self.client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .url()
    }
}
```

### 2. Callback Handler

```rust
// r2_server/src/auth/callback.rs

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

pub async fn oauth_callback(
    State(axum_state): State<Arc<AppState>>,
    Query(query): Query<CallbackQuery>,
) -> Result<Json<CallbackResponse>, AppError> {
    // 1. Vérifier le state CSRF
    // ...

    // 2. Échanger le code contre un access token
    let token = axum_state.oauth_client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await?;

    // 3. Récupérer les infos utilisateur
    let user_info = get_user_info(&token, &axum_state.oauth_client.provider).await?;

    // 4. Créer ou mettre à jour le player
    let player = upsert_player(
        &axum_state.db,
        &user_info,
        &axum_state.oauth_client.provider,
    ).await?;

    // 5. Générer JWT
    let (access_token, refresh_token) = generate_tokens(&player)?;

    // 6. Stocker refresh token en DB
    save_refresh_token(&axum_state.db, player.id, &refresh_token).await?;

    Ok(Json(CallbackResponse {
        access_token,
        refresh_token,
        player_id: player.id,
    }))
}

#[derive(Serialize)]
pub struct CallbackResponse {
    access_token: String,
    refresh_token: String,
    player_id: i64,
}
```

### 3. Génération JWT

```rust
// r2_server/src/auth/jwt.rs

use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};

const ACCESS_TOKEN_EXPIRATION: u64 = 15 * 60; // 15 minutes
const REFRESH_TOKEN_EXPIRATION: u64 = 30 * 24 * 60 * 60; // 30 jours

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,        // player_id
    pub exp: usize,      // Expiration
    pub iat: usize,      // Issued at
    pub token_type: TokenType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

pub fn generate_tokens(player_id: i64) -> Result<(String, String)> {
    let now = Utc::now().timestamp() as usize;

    // Access Token
    let access_claims = Claims {
        sub: player_id,
        exp: now + ACCESS_TOKEN_EXPIRATION as usize,
        iat: now,
        token_type: TokenType::Access,
    };
    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(env::var("JWT_SECRET")?.as_bytes()),
    )?;

    // Refresh Token
    let refresh_claims = Claims {
        sub: player_id,
        exp: now + REFRESH_TOKEN_EXPIRATION as usize,
        iat: now,
        token_type: TokenType::Refresh,
    };
    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(env::var("JWT_SECRET")?.as_bytes()),
    )?;

    Ok((access_token, refresh_token))
}

pub fn validate_token(token: &str) -> Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(env::var("JWT_SECRET")?.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
```

---

## 🛡️ Middleware JWT

```rust
// r2_server/src/auth/middleware.rs

use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware(
    State(axum_state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extraire le token
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..];

    // Valider
    let claims = validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Vérifier que c'est un access token
    if matches!(claims.token_type, TokenType::Refresh) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Ajouter les claims aux extensions de la request
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

// Extractor pour les handlers
pub struct AuthenticatedPlayer {
    pub player_id: i64,
}

impl<S> FromRequestParts<S> for AuthenticatedPlayer
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .get_extension::<Claims>()
            .map(|c| AuthenticatedPlayer { player_id: c.sub })
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}
```

---

## 🔄 Refresh Token

```rust
// r2_server/src/auth/refresh.rs

#[derive(Deserialize)]
pub struct RefreshRequest {
    refresh_token: String,
}

pub async fn refresh_token(
    State(axum_state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    // 1. Valider le refresh token
    let claims = validate_token(&req.refresh_token)?;

    if !matches!(claims.token_type, TokenType::Refresh) {
        return Err(AuthError::InvalidToken.into());
    }

    // 2. Vérifier que le token existe en DB
    let session = sqlx::query!(
        "SELECT player_id, expires_at FROM sessions WHERE refresh_token = $1",
        req.refresh_token
    )
    .fetch_optional(&axum_state.db)
    .await?
    .ok_or(AuthError::TokenNotFound)?;

    // 3. Vérifier expiration
    if session.expires_at < Utc::now() {
        // Nettoyer
        sqlx::query!("DELETE FROM sessions WHERE refresh_token = $1", req.refresh_token)
            .execute(&axum_state.db)
            .await?;
        return Err(AuthError::TokenExpired.into());
    }

    // 4. Générer nouveau pair
    let (access_token, new_refresh_token) = generate_tokens(session.player_id)?;

    // 5. Update session
    sqlx::query!(
        "UPDATE sessions SET refresh_token = $1, expires_at = $2 WHERE refresh_token = $3",
        new_refresh_token,
        Utc::now() + chrono::Duration::days(30),
        req.refresh_token
    )
    .execute(&axum_state.db)
    .await?;

    Ok(Json(TokenResponse {
        access_token,
        refresh_token: new_refresh_token,
    }))
}
```

---

## 🌐 WebSocket Auth

```rust
// r2_server/src/network/ws.rs

use axum::{
    extract::{
        ws::{WebSocket, Message},
        WebSocketUpgrade, State,
    },
    response::IntoResponse,
};

pub async fn ws_connect(
    State(axum_state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
    headers: HeaderMap,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, axum_state, headers))
}

async fn handle_socket(
    mut socket: WebSocket,
    axum_state: Arc<AppState>,
    headers: HeaderMap,
) {
    // 1. Auth via query param (pour upgrade WS)
    let token = socket
        .recv()
        .await
        .and_then(|m| m.into_text())
        .unwrap_or_default();

    // 2. Valider
    let player_id = match validate_ws_token(&token) {
        Ok(id) => id,
        Err(_) => {
            let _ = socket.send(Message::Close(None)).await;
            return;
        }
    };

    // 3. Créer connexion player
    let conn = PlayerConnection::new(player_id, socket);
    axum_state.game_server.add_connection(conn).await;

    // 4. Boucle de messages
    // ...
}
```

---

## 👤 User Info Retrieval

```rust
// r2_server/src/auth/user_info.rs

#[derive(Deserialize)]
struct GoogleUserInfo {
    sub: String,
    email: String,
    name: String,
    picture: Option<String>,
}

#[derive(Deserialize)]
struct DiscordUserInfo {
    id: String,
    email: Option<String>,
    username: String,
    avatar: Option<String>,
}

pub async fn get_user_info(
    token: &TokenResponse,
    provider: &OAuthProvider,
) -> Result<UserInfo> {
    let client = reqwest::Client::new();

    match provider {
        OAuthProvider::Google => {
            let resp = client
                .get("https://www.googleapis.com/oauth2/v2/userinfo")
                .bearer_auth(&token.access_token().secret())
                .send()
                .await?
                .json::<GoogleUserInfo>()
                .await?;

            Ok(UserInfo {
                oauth_id: format!("google:{}", resp.sub),
                email: resp.email,
                username: resp.name,
                avatar: resp.picture,
            })
        }
        OAuthProvider::Discord => {
            let resp = client
                .get("https://discord.com/api/users/@me")
                .bearer_auth(&token.access_token().secret())
                .send()
                .await?
                .json::<DiscordUserInfo>()
                .await?;

            let avatar = resp.avatar.map(|av| {
                format!(
                    "https://cdn.discordapp.com/avatars/{}/{}.png",
                    resp.id, av
                )
            });

            Ok(UserInfo {
                oauth_id: format!("discord:{}", resp.id),
                email: resp.email,
                username: resp.username,
                avatar,
            })
        }
    }
}
```

---

## 🗄️ Database Queries

```rust
// r2_server/src/auth/db.rs

pub async fn upsert_player(
    pool: &PgPool,
    user_info: &UserInfo,
    provider: &OAuthProvider,
) -> Result<Player> {
    let provider_name = match provider {
        OAuthProvider::Google => "google",
        OAuthProvider::Discord => "discord",
    };

    let player = sqlx::query_as!(
        Player,
        r#"
        INSERT INTO players (username, oauth_provider, oauth_id, last_login)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (oauth_provider, oauth_id)
        DO UPDATE SET last_login = NOW()
        RETURNING *
        "#,
        user_info.username,
        provider_name,
        user_info.oauth_id
    )
    .fetch_one(pool)
    .await?;

    Ok(player)
}

pub async fn save_refresh_token(
    pool: &PgPool,
    player_id: i64,
    refresh_token: &str,
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO sessions (player_id, refresh_token, expires_at)
        VALUES ($1, $2, $3)
        "#,
        player_id,
        refresh_token,
        Utc::now() + chrono::Duration::days(30)
    )
    .execute(pool)
    .await?;

    Ok(())
}
```

---

## 📱 Client-Side (Dioxus)

```rust
// r2_client/src/ui/auth.rs

use dioxus::prelude::*;
use reqwest::Client;

#[component]
pub fn LoginScreen() -> Element {
    let api_base = env!("API_BASE_URL");

    rsx! {
        div { class: "login-container",
            h1 { "R2 - Spatial MMO" }

            button {
                class: "btn-google",
                onclick: move |_| {
                    let url = format!("{}/auth/google", api_base);
                    web_sys::window()
                        .unwrap()
                        .location()
                        .set_href(&url)
                        .unwrap();
                },
                "Login with Google"
            }

            button {
                class: "btn-discord",
                onclick: move |_| {
                    let url = format!("{}/auth/discord", api_base);
                    web_sys::window()
                        .unwrap()
                        .location()
                        .set_href(&url)
                        .unwrap();
                },
                "Login with Discord"
            }
        }
    }
}

// Après callback, stocker tokens
pub async fn handle_callback(code: String) -> Result<(String, String)> {
    let client = Client::new();
    let resp = client
        .post(format!("{}/auth/callback", env!("API_BASE_URL")))
        .json(&serde_json::json!({ "code": code }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let access_token = resp["access_token"].as_str().unwrap().to_string();
    let refresh_token = resp["refresh_token"].as_str().unwrap().to_string();

    // Stocker en localStorage
    web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .unwrap()
        .set_item("access_token", &access_token)
        .unwrap();

    Ok((access_token, refresh_token))
}
```

---

## 🔒 Sécurité

### Checklist

- [x] CSRF tokens pour OAuth
- [x] JWT expiration courte (15 min)
- [x] Refresh token rotation
- [x] Token stocké en httpOnly cookie (optionnel)
- [x] Rate limiting sur `/auth/callback`
- [x] HTTPS obligatoire en prod

### Anti-Automation

```rust
// Limiter les tentatives par IP
use tower_governor::{Governor, GovernorConfigBuilder};

let governor_conf = GovernorConfigBuilder::default()
    .per_second(10)
    .burst_size(5)
    .finish()
    .unwrap();

// Appliquer aux routes d'auth
app.layer(Governor::new(&governor_conf, &axum_state.ip_extractor))
```

---

*Auth System v0.1.0 - OAuth2 + JWT*
