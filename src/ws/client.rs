//! WebSocket client for Godot plugin communication
//!
//! Provides a simple WebSocket client that connects, sends a command, receives a response,
//! and disconnects. This follows the same lifecycle as the HTTP implementation.

use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// WebSocket client errors
#[derive(Debug, thiserror::Error)]
pub enum WsError {
    #[error("Failed to connect: {0}")]
    ConnectionFailed(String),

    #[error("Failed to send message: {0}")]
    SendFailed(String),

    #[error("Failed to receive response: {0}")]
    ReceiveFailed(String),

    #[error("Connection timeout")]
    Timeout,

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// WebSocket client for communicating with Godot plugin
pub struct WsClient {
    url: String,
    timeout_duration: Duration,
}

impl WsClient {
    /// Create a new WebSocket client targeting the specified port
    pub fn new(port: u16) -> Self {
        Self {
            url: format!("ws://localhost:{}", port),
            timeout_duration: Duration::from_secs(5),
        }
    }

    /// Set custom timeout duration
    pub fn with_timeout(mut self, duration: Duration) -> Self {
        self.timeout_duration = duration;
        self
    }

    /// Send a command and receive a response
    ///
    /// This method:
    /// 1. Establishes a WebSocket connection
    /// 2. Sends the JSON-serialized command
    /// 3. Waits for a response
    /// 4. Closes the connection
    /// 5. Returns the response text
    pub async fn send_command<T: Serialize>(&self, command: &T) -> Result<String, WsError> {
        // Serialize command to JSON
        let json = serde_json::to_string(command)
            .map_err(|e| WsError::SerializationError(e.to_string()))?;

        // Connect with timeout
        let connect_future = connect_async(&self.url);
        let (ws_stream, _response) = timeout(self.timeout_duration, connect_future)
            .await
            .map_err(|_| WsError::Timeout)?
            .map_err(|e| WsError::ConnectionFailed(e.to_string()))?;

        let (mut write, mut read) = ws_stream.split();

        // Send command
        write
            .send(Message::Text(json))
            .await
            .map_err(|e| WsError::SendFailed(e.to_string()))?;

        // Wait for response with timeout
        let response = timeout(self.timeout_duration, read.next())
            .await
            .map_err(|_| WsError::Timeout)?
            .ok_or_else(|| WsError::ReceiveFailed("Connection closed".to_string()))?
            .map_err(|e| WsError::ReceiveFailed(e.to_string()))?;

        // Extract text from response
        match response {
            Message::Text(text) => Ok(text),
            Message::Binary(data) => String::from_utf8(data)
                .map_err(|e| WsError::ReceiveFailed(format!("Invalid UTF-8: {}", e))),
            Message::Close(_) => Err(WsError::ReceiveFailed("Connection closed".to_string())),
            _ => Err(WsError::ReceiveFailed(
                "Unexpected message type".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_client_url_format() {
        let client = WsClient::new(6061);
        assert_eq!(client.url, "ws://localhost:6061");
    }

    #[test]
    fn test_ws_client_with_timeout() {
        let client = WsClient::new(6061).with_timeout(Duration::from_secs(10));
        assert_eq!(client.timeout_duration, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_ws_client_connection_refused() {
        // Test that connection to non-existent server fails gracefully
        let client = WsClient::new(59999); // Use a port that's unlikely to be in use
        let result = client
            .send_command(&serde_json::json!({"command": "ping"}))
            .await;
        assert!(result.is_err());

        match result {
            Err(WsError::ConnectionFailed(_)) => {} // Expected
            Err(WsError::Timeout) => {}             // Also acceptable
            other => panic!("Unexpected result: {:?}", other),
        }
    }
}
