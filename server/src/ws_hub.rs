use std::collections::HashSet;
use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct WsHub {
    connections: Arc<DashMap<String, mpsc::UnboundedSender<String>>>,
    sessions: Arc<DashMap<String, SessionChannel>>,
}

pub struct SessionChannel {
    pub participants: HashSet<String>,
    pub owner: String,
}

impl Default for WsHub {
    fn default() -> Self {
        Self::new()
    }
}

impl WsHub {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
            sessions: Arc::new(DashMap::new()),
        }
    }

    pub fn register(&self, token_id: String, tx: mpsc::UnboundedSender<String>) {
        self.connections.insert(token_id, tx);
    }

    pub fn unregister(&self, token_id: &str) -> Vec<String> {
        self.connections.remove(token_id);

        let mut paused_sessions = Vec::new();
        for entry in self.sessions.iter() {
            if entry.value().owner == token_id {
                paused_sessions.push(entry.key().to_string());
            }
        }
        paused_sessions
    }

    pub fn register_session(
        &self,
        session_id: String,
        owner: String,
        participants: HashSet<String>,
    ) {
        self.sessions.insert(
            session_id,
            SessionChannel {
                participants,
                owner,
            },
        );
    }

    pub fn add_session_participant(&self, session_id: &str, token_id: String) {
        if let Some(mut channel) = self.sessions.get_mut(session_id) {
            channel.participants.insert(token_id);
        }
    }

    pub fn remove_session_participant(&self, session_id: &str, token_id: &str) {
        if let Some(mut channel) = self.sessions.get_mut(session_id) {
            channel.participants.remove(token_id);
        }
    }

    pub fn remove_session(&self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    pub fn broadcast_to_session(&self, session_id: &str, message: &str) {
        if let Some(channel) = self.sessions.get(session_id) {
            for participant in &channel.participants {
                if let Some(tx) = self.connections.get(participant) {
                    let _ = tx.send(message.to_string());
                }
            }
        }
    }

    pub fn send_to_user(&self, token_id: &str, message: &str) {
        if let Some(tx) = self.connections.get(token_id) {
            let _ = tx.send(message.to_string());
        }
    }

    pub fn sessions_owned_by(&self, token_id: &str) -> Vec<String> {
        self.sessions
            .iter()
            .filter(|entry| entry.value().owner == token_id)
            .map(|entry| entry.key().to_string())
            .collect()
    }

    pub fn is_session_owner_online(&self, session_id: &str) -> bool {
        if let Some(channel) = self.sessions.get(session_id) {
            self.connections.contains_key(&channel.owner)
        } else {
            false
        }
    }
}
