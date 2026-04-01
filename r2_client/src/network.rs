//! Module réseau côté client

use r2_core::network::R2Message;
use serde_json;

/// Client WebSocket pour R2
pub struct R2WebSocketClient {
    // TODO: Implémenter quand tokio-tungstenite sera configuré
}

impl R2WebSocketClient {
    pub fn new(_url: String) -> Self {
        Self { }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implémenter la connexion WebSocket
        Ok(())
    }

    pub async fn send(&mut self, msg: &R2Message) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(msg)?;
        tracing::debug!("Sending: {}", json);
        // TODO: Envoyer via WebSocket
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Option<R2Message>, Box<dyn std::error::Error>> {
        // TODO: Recevoir via WebSocket
        Ok(None)
    }
}
