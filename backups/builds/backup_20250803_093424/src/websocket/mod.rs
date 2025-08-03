//! WebSocket Integration Module
//! 
//! This module provides WebSocket functionality for real-time communication,
//! remote sessions, and collaborative features.

pub mod client;
pub mod server;
pub mod connection;
pub mod protocol;
pub mod handlers;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    pub server_port: u16,
    pub max_connections: usize,
    pub enable_tls: bool,
    pub auth_required: bool,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            server_port: 8080,
            max_connections: 100,
            enable_tls: false,
            auth_required: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebSocketMessage {
    Command { id: String, command: String },
    Output { id: String, output: String },
    Status { status: String },
    Error { error: String },
}

pub type ConnectionId = String;
pub type SessionId = String;

pub struct WebSocketManager {
    connections: HashMap<ConnectionId, SessionId>,
    config: WebSocketConfig,
}

impl WebSocketManager {
    pub fn new(config: WebSocketConfig) -> Self {
        Self {
            connections: HashMap::new(),
            config,
        }
    }

    pub fn add_connection(&mut self, connection_id: ConnectionId, session_id: SessionId) {
        self.connections.insert(connection_id, session_id);
    }

    pub fn remove_connection(&mut self, connection_id: &ConnectionId) {
        self.connections.remove(connection_id);
    }

    pub fn get_session(&self, connection_id: &ConnectionId) -> Option<&SessionId> {
        self.connections.get(connection_id)
    }
}
