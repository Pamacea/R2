//! Module réseau pour R2 Server
//!
//! Gère les connexions WebSocket et les messages.

#![allow(dead_code)]  // TODO: À utiliser dans les prochaines versions

use crate::auth::validate_access_token;
use crate::AppState;
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

// ============================================================================
// MESSAGES CLIENT → SERVEUR
// ============================================================================

/// Message envoyé par le client vers le serveur
/// Format compatible avec le client JavaScript: {type: "...", ...}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Connexion avec token JWT
    #[serde(rename = "connect")]
    Connect { token: String },

    /// Input du joueur (contrôles en temps réel)
    #[serde(rename = "input")]
    Input {
        thrust: bool,
        rotate: f32,
        fire: bool,
    },

    /// Ping pour keep-alive
    #[serde(rename = "ping")]
    Ping { timestamp: i64 },
}

// ============================================================================
// MESSAGES SERVEUR → CLIENT
// ============================================================================

/// Message envoyé par le serveur vers le client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Réponse à la connexion réussie
    #[serde(rename = "connected")]
    Connected {
        player_id: i64,
        characters: Vec<CharacterSummary>,
    },

    /// Mise à jour de l'état du monde (AOI)
    #[serde(rename = "worldUpdate")]
    WorldUpdate {
        entities: Vec<EntityData>,
        timestamp: i64,
    },

    /// Erreur
    #[serde(rename = "error")]
    Error { message: String },

    /// Pong réponse
    #[serde(rename = "pong")]
    Pong { timestamp: i64 },
}

// ============================================================================
// TYPES DE DONNÉES
// ============================================================================

/// Résumé d'un personnage pour la sélection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSummary {
    pub id: i64,
    pub name: String,
    pub ship_class: String,
    pub level: u8,
}

/// Données d'une entité pour les mises à jour monde
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityData {
    pub id: u64,
    pub entity_type: EntityType,
    pub position: (f32, f32, f32),
    pub rotation: f32,
    pub velocity: Option<(f32, f32)>,
    pub hull_percent: f32,
    pub shield_percent: f32,
}

/// Type d'entité
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EntityType {
    Player,
    NpcHostile,
    NpcFriendly,
    Asteroid,
    Station,
    Loot,
    Projectile,
}

// ============================================================================
// CONNEXION JOUEUR
// ============================================================================

/// Connexion WebSocket d'un joueur
pub struct PlayerConnection {
    player_id: Option<i64>,
    character_id: Option<i64>,
    position: (f32, f32, f32),
}

impl PlayerConnection {
    pub fn new() -> Self {
        Self {
            player_id: None,
            character_id: None,
            position: (0.0, 0.0, 0.0),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.player_id.is_some()
    }
}

impl Default for PlayerConnection {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// GESTIONNAIRE DE CONNEXIONS (POUR BROADCAST)
// ============================================================================

/// Sender vers un client spécifique
pub type ClientSender = broadcast::Sender<ServerMessage>;

/// Informations sur un client connecté
#[derive(Clone)]
struct ConnectedClient {
    player_id: i64,
    sender: ClientSender,
    position: (f32, f32),
}

/// Gestionnaire des connexions actives pour le broadcast
#[derive(Clone)]
pub struct ConnectionManager {
    clients: Arc<RwLock<Vec<ConnectedClient>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Ajouter un client connecté
    pub async fn add_client(&self, player_id: i64) -> ClientSender {
        let (tx, _rx) = broadcast::channel(100);
        let client = ConnectedClient {
            player_id,
            sender: tx.clone(),
            position: (0.0, 0.0),
        };
        self.clients.write().await.push(client);
        tx
    }

    /// Retirer un client
    pub async fn remove_client(&self, player_id: i64) {
        self.clients
            .write()
            .await
            .retain(|c| c.player_id != player_id);
    }

    /// Mettre à jour la position d'un client
    pub async fn update_position(&self, player_id: i64, x: f32, y: f32) {
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.iter_mut().find(|c| c.player_id == player_id) {
            client.position = (x, y);
        }
    }

    /// Obtenir les clients dans une zone (AOI)
    pub async fn get_clients_in_radius(&self, x: f32, y: f32, radius: f32) -> Vec<i64> {
        let clients = self.clients.read().await;
        clients
            .iter()
            .filter(|c| {
                let dx = c.position.0 - x;
                let dy = c.position.1 - y;
                (dx * dx + dy * dy).sqrt() <= radius
            })
            .map(|c| c.player_id)
            .collect()
    }

    /// Envoyer un message à tous les clients
    pub async fn broadcast(&self, message: &ServerMessage) {
        let clients = self.clients.read().await;
        for client in clients.iter() {
            let _ = client.sender.send(message.clone());
        }
    }

    /// Envoyer un message à un client spécifique
    pub async fn send_to(&self, player_id: i64, message: &ServerMessage) -> bool {
        let clients = self.clients.read().await;
        if let Some(client) = clients.iter().find(|c| c.player_id == player_id) {
            client.sender.send(message.clone()).is_ok()
        } else {
            false
        }
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// HANDLER WEBSOCKET
// ============================================================================

/// Handler pour une connexion WebSocket
pub async fn handle_websocket(
    socket: WebSocket,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("🔌 New WebSocket connection");

    // Split en sender/receiver
    let (mut sender, mut receiver) = socket.split();

    // Première attente: auth via JWT
    let mut conn = PlayerConnection::new();
    let mut connection_manager = ConnectionManager::new();
    let mut client_sender: Option<ClientSender> = None;

    // Boucle de messages
    while let Some(msg_result) = receiver.next().await {
        match msg_result {
            Ok(msg) => {
                match handle_message(msg, &mut sender, &mut conn, &state, &mut connection_manager, &mut client_sender).await {
                    Ok(should_continue) => {
                        if !should_continue {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error handling message: {}", e);
                        let error_msg = ServerMessage::Error {
                            message: format!("Error: {}", e),
                        };
                        let _ = send_server_message(&mut sender, &error_msg).await;
                        break;
                    }
                }
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Nettoyage à la déconnexion
    if let Some(pid) = conn.player_id {
        info!("Player {} disconnected", pid);
        connection_manager.remove_client(pid).await;
    }

    info!("🔌 WebSocket connection closed");
    Ok(())
}

/// Traiter un message WebSocket
async fn handle_message(
    msg: Message,
    sender: &mut futures_util::stream::SplitSink<axum::extract::ws::WebSocket, Message>,
    conn: &mut PlayerConnection,
    state: &AppState,
    conn_manager: &ConnectionManager,
    client_sender: &mut Option<ClientSender>,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    match msg {
        Message::Text(text) => {
            debug!("Received text: {}", text);

            // Parser le message client
            let client_msg: ClientMessage = match serde_json::from_str(&text) {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Failed to parse message: {}", e);
                    send_server_message(sender, &ServerMessage::Error {
                        message: format!("Invalid message format: {}", e),
                    }).await?;
                    return Ok(true);
                }
            };

            // Traiter selon le type
            match client_msg {
                ClientMessage::Connect { token } => {
                    // Valider le token JWT
                    match validate_access_token(&token) {
                        Ok(player_id) => {
                            info!("✅ Player {} authenticated", player_id);
                            conn.player_id = Some(player_id);

                            // Créer le sender pour ce client
                            let tx = conn_manager.add_client(player_id).await;
                            *client_sender = Some(tx);

                            // Envoyer la réponse de connexion
                            let response = ServerMessage::Connected {
                                player_id,
                                characters: vec![],
                            };
                            send_server_message(sender, &response).await?;
                        }
                        Err(e) => {
                            warn!("❌ Authentication failed: {}", e);
                            send_server_message(sender, &ServerMessage::Error {
                                message: format!("Authentication failed: {}", e),
                            }).await?;
                            return Ok(false); // Fermer la connexion
                        }
                    }
                }
                ClientMessage::Input { thrust, rotate, fire } => {
                    // TODO: Gérer l'input du joueur
                    debug!("Input: thrust={}, rotate={}, fire={}", thrust, rotate, fire);
                }
                ClientMessage::Ping { timestamp } => {
                    // Répondre avec un pong
                    let pong = ServerMessage::Pong { timestamp };
                    send_server_message(sender, &pong).await?;
                }
            }

            Ok(true)
        }
        Message::Close(_) => {
            info!("Client initiated close");
            Ok(false)
        }
        _ => Ok(true),
    }
}

/// Envoyer un message au client
async fn send_server_message(
    sender: &mut futures_util::stream::SplitSink<axum::extract::ws::WebSocket, Message>,
    msg: &ServerMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let json = serde_json::to_string(msg)?;
    sender.send(Message::Text(json)).await?;
    Ok(())
}

// ============================================================================
// EXPORTS
// ============================================================================

pub type WebSocketState = Arc<RwLock<ConnectionManager>>;
