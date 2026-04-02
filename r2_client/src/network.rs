//! Module réseau côté client
//!
//! Gère la connexion WebSocket avec le serveur de jeu.

use bevy::prelude::*;
use r2_core::network::*;
use serde_json;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Erreur réseau client
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Send error: {0}")]
    SendError(String),

    #[error("Receive error: {0}")]
    ReceiveError(String),

    #[error("Not connected")]
    NotConnected,
}

/// Type de résultat pour les opérations client
pub type ClientResult<T> = Result<T, ClientError>;

/// Configuration du client WebSocket
#[derive(Clone, Debug)]
pub struct ClientConfig {
    /// URL du serveur WebSocket (ex: ws://localhost:3000/ws)
    pub server_url: String,
    /// Token d'authentification JWT
    pub auth_token: Option<String>,
    /// Taille du buffer de réception
    pub recv_buffer_size: usize,
    /// Timeout de connexion en secondes
    pub connect_timeout: u64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_url: "ws://localhost:3000/ws".to_string(),
            auth_token: None,
            recv_buffer_size: 1000,
            connect_timeout: 10,
        }
    }
}

impl ClientConfig {
    /// Crée une nouvelle configuration
    pub fn new(server_url: String) -> Self {
        Self {
            server_url,
            ..Default::default()
        }
    }

    /// Définit le token d'authentification
    pub fn with_auth(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    /// Construit l'URL complète avec le token si présent
    pub fn build_url(&self) -> String {
        if let Some(token) = &self.auth_token {
            format!("{}?token={}", self.server_url, token)
        } else {
            self.server_url.clone()
        }
    }
}

/// Message réseau reçu avec son timestamp
#[derive(Clone, Debug)]
pub struct NetworkMessage {
    /// Le message R2
    pub message: R2Message,
    /// Timestamp de réception (seconds since epoch)
    pub timestamp: f64,
}

impl NetworkMessage {
    /// Crée un nouveau message réseau
    pub fn new(message: R2Message) -> Self {
        Self {
            message,
            timestamp: chrono::Utc::now().timestamp_millis() as f64 / 1000.0,
        }
    }
}

/// État de la connexion
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error,
}

/// Client WebSocket pour R2
pub struct R2WebSocketClient {
    /// Configuration du client
    config: ClientConfig,
    /// État de la connexion
    state: Arc<Mutex<ConnectionState>>,
    /// Channel pour envoyer des messages au handler de réception
    tx: Option<mpsc::UnboundedSender<NetworkMessage>>,
    /// Channel pour les messages sortants
    outgoing_tx: Option<mpsc::UnboundedSender<R2Message>>,
}

impl R2WebSocketClient {
    /// Crée un nouveau client
    pub fn new(config: ClientConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(ConnectionState::Disconnected)),
            tx: None,
            outgoing_tx: None,
        }
    }

    /// Retourne l'état actuel de la connexion
    pub async fn state(&self) -> ConnectionState {
        *self.state.lock().await
    }

    /// Connecte au serveur WebSocket
    pub async fn connect(&mut self) -> ClientResult<()> {
        *self.state.lock().await = ConnectionState::Connecting;

        let url = self.config.build_url();
        let url_for_log = url.clone();

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.connect_native(url).await?;
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.connect_wasm(url).await?;
        }

        *self.state.lock().await = ConnectionState::Connected;
        tracing::info!("Connected to game server at {}", url_for_log);

        Ok(())
    }

    /// Connexion native (non-WASM)
    #[cfg(not(target_arch = "wasm32"))]
    async fn connect_native(&mut self, url: String) -> ClientResult<()> {
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::{connect_async, tungstenite::Message};

        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| ClientError::ConnectionFailed(e.to_string()))?;

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Channel pour les messages entrants
        let (tx, _rx) = mpsc::unbounded_channel();
        self.tx = Some(tx.clone());

        // Channel pour les messages sortants
        let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel();
        self.outgoing_tx = Some(outgoing_tx);

        // Task de réception
        let state = self.state.clone();
        tokio::spawn(async move {
            while let Some(msg) = ws_receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(r2_msg) = serde_json::from_str::<R2Message>(&text) {
                            let _ = tx.send(NetworkMessage::new(r2_msg));
                        }
                    }
                    Ok(Message::Close(_)) => {
                        *state.lock().await = ConnectionState::Disconnected;
                        break;
                    }
                    Err(e) => {
                        tracing::error!("WebSocket error: {}", e);
                        *state.lock().await = ConnectionState::Error;
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Task d'envoi
        tokio::spawn(async move {
            while let Some(msg) = outgoing_rx.recv().await {
                let json = match serde_json::to_string(&msg) {
                    Ok(j) => j,
                    Err(_) => continue,
                };

                if ws_sender
                    .send(Message::Text(json))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        Ok(())
    }

    /// Connexion WASM (navigateur)
    #[cfg(target_arch = "wasm32")]
    async fn connect_wasm(&mut self, url: String) -> ClientResult<()> {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{WebSocket, MessageEvent};

        // Créer la connexion WebSocket
        let ws = WebSocket::new(&url)
            .map_err(|e| ClientError::ConnectionFailed(format!("{:?}", e)))?;

        // Configurer pour les messages binaires
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        // Channels
        let (tx, _rx) = mpsc::unbounded_channel();
        let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel();

        self.tx = Some(tx.clone());
        self.outgoing_tx = Some(outgoing_tx);

        // Clones pour les callbacks
        let ws_clone = ws.clone();
        let state = self.state.clone();

        // Callback onmessage
        let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Some(text) = event.data().as_string() {
                if let Ok(r2_msg) = serde_json::from_str::<R2Message>(&text) {
                    let _ = tx.send(NetworkMessage::new(r2_msg));
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget(); // Important: ne pas drop le callback

        // Callback onopen
        let state_clone = state.clone();
        let onopen_callback = Closure::wrap(Box::new(move |_| {
            *state_clone.lock().await = ConnectionState::Connected;
        }) as Box<dyn FnMut(JsValue)>);

        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        // Callback onerror
        let state_clone = state.clone();
        let onerror_callback = Closure::wrap(Box::new(move |_| {
            *state_clone.lock().await = ConnectionState::Error;
        }) as Box<dyn FnMut(JsValue)>);

        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        // Task d'envoi pour WASM
        tokio::spawn(async move {
            while let Some(msg) = outgoing_rx.recv().await {
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = ws_clone.send_with_str(&json);
                }
            }
        });

        Ok(())
    }

    /// Envoie un message au serveur
    pub async fn send(&mut self, msg: R2Message) -> ClientResult<()> {
        if let Some(tx) = &self.outgoing_tx {
            tx.send(msg)
                .map_err(|_| ClientError::SendError("Channel closed".to_string()))?;
            Ok(())
        } else {
            Err(ClientError::NotConnected)
        }
    }

    /// Essaye de recevoir un message (non-bloquant)
    pub fn try_recv(&self) -> ClientResult<Option<R2Message>> {
        // Note: en pratique, on utilisera un channel dans un système Bevy
        // Le receiver est exposé via WebSocketResource
        Ok(None)
    }

    /// Déconnecte du serveur
    pub async fn disconnect(&mut self) {
        *self.state.lock().await = ConnectionState::Disconnected;
        self.tx = None;
        self.outgoing_tx = None;
    }
}

/// Ressource Bevy pour le client WebSocket
#[derive(Resource)]
pub struct WebSocketResource {
    /// Le client WebSocket
    pub client: Arc<Mutex<R2WebSocketClient>>,
    /// Channel pour recevoir les messages (pour les systèmes Bevy)
    pub message_rx: mpsc::UnboundedReceiver<NetworkMessage>,
}

impl WebSocketResource {
    /// Crée une nouvelle ressource WebSocket
    pub fn new(client: R2WebSocketClient, message_rx: mpsc::UnboundedReceiver<NetworkMessage>) -> Self {
        Self {
            client: Arc::new(Mutex::new(client)),
            message_rx,
        }
    }
}

/// Plugin réseau pour Bevy
pub struct NetworkPlugin {
    /// Configuration du client
    pub config: ClientConfig,
}

impl NetworkPlugin {
    /// Crée un nouveau plugin avec la configuration donnée
    pub fn new(config: ClientConfig) -> Self {
        Self { config }
    }
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_network);
    }
}

/// Setup du réseau
fn setup_network() {
    // Le client sera initialisé plus tard avec la configuration
    tracing::info!("Network plugin initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_build_url() {
        let config = ClientConfig::new("ws://localhost:3000/ws".to_string())
            .with_auth("test_token".to_string());

        let url = config.build_url();
        assert!(url.contains("token=test_token"));
    }

    #[test]
    fn test_client_creation() {
        let config = ClientConfig::default();
        let client = R2WebSocketClient::new(config);
        // Le client devrait être créé sans erreur
    }
}
