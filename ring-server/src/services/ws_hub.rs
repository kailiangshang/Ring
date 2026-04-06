use std::collections::HashMap;

use serde::Serialize;
use tokio::sync::{broadcast, RwLock};

#[derive(Debug, Clone, Serialize)]
pub struct WsMessage {
    pub msg_type: String,
    pub payload: serde_json::Value,
}

pub struct WsHub {
    channels: RwLock<HashMap<String, broadcast::Sender<WsMessage>>>,
}

impl Default for WsHub {
    fn default() -> Self {
        Self::new()
    }
}

impl WsHub {
    pub fn new() -> Self {
        WsHub {
            channels: RwLock::new(HashMap::new()),
        }
    }

    pub async fn subscribe(&self, ring_id: &str) -> broadcast::Receiver<WsMessage> {
        let mut channels = self.channels.write().await;
        let tx = channels
            .entry(ring_id.to_string())
            .or_insert_with(|| broadcast::channel(256).0);
        tx.subscribe()
    }

    pub async fn broadcast(&self, ring_id: &str, msg: WsMessage) {
        let channels = self.channels.read().await;
        if let Some(tx) = channels.get(ring_id) {
            let _ = tx.send(msg);
        }
    }
}
