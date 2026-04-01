//! Module réseau pour R2 Server
//!
//! Gère les connexions WebSocket et les messages.

use crate::AppState;
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use r2_core::network::R2Message;
use serde_json;
use tracing::{debug, error, info};

/// Connexion WebSocket d'un joueur
pub struct PlayerConnection {
    player_id: Option<i64>,
    character_id: Option<i64>,
}

impl PlayerConnection {
    pub fn new() -> Self {
        Self {
            player_id: None,
            character_id: None,
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

/// Handler pour une connexion WebSocket
pub async fn handle_websocket(
    mut socket: WebSocket,
    _state: AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔌 New WebSocket connection");

    // Première attente: auth via JWT
    let mut conn = PlayerConnection::new();

    // Boucle de messages
    while let Some(msg_result) = socket.recv().await {
        match msg_result {
            Ok(msg) => {
                if !handle_message(msg, &mut socket, &mut conn).await? {
                    break;
                }
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    info!("🔌 WebSocket connection closed");

    Ok(())
}

/// Traiter un message WebSocket
async fn handle_message(
    msg: Message,
    socket: &mut WebSocket,
    _conn: &mut PlayerConnection,
) -> Result<bool, Box<dyn std::error::Error>> {
    match msg {
        Message::Text(text) => {
            debug!("Received text: {}", text);

            // Parser le message
            let r2_msg: R2Message = match serde_json::from_str(&text) {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Failed to parse message: {}", e);
                    // TODO: envoyer erreur
                    return Ok(true);
                }
            };

            // Traiter selon le type
            match r2_msg {
                R2Message::Ping { timestamp } => {
                    let pong = R2Message::Pong { timestamp };
                    let pong_json = serde_json::to_string(&pong)?;
                    socket.send(Message::Text(pong_json)).await?;
                }

                R2Message::MoveTo { target } => {
                    debug!("MoveTo: x={}, y={}", target.x, target.y);
                    // TODO: traiter le mouvement
                }

                R2Message::UseSkill { slot, target_id } => {
                    debug!("UseSkill: slot={}, target={:?}", slot, target_id);
                    // TODO: traiter l'utilisation de skill
                }

                R2Message::Chat { channel, message } => {
                    debug!("Chat: {:?}: {}", channel, message);
                    // TODO: traiter le chat
                }

                _ => {
                    debug!("Unhandled message type");
                }
            }

            Ok(true)
        }

        Message::Close(_) => {
            info!("Client initiated close");
            Ok(false)
        }

        Message::Ping(data) => {
            socket.send(Message::Pong(data)).await?;
            Ok(true)
        }

        _ => {
            // Ignorer les autres types de messages
            Ok(true)
        }
    }
}
