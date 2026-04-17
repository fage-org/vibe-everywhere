//! Pairing Flow Module
//!
//! Handles the initial pairing process when the daemon has no credentials.

use std::sync::Arc;
use std::time::Duration;

use rand::Rng;
use reqwest::Client;
use tokio::sync::broadcast;
use tracing::{info, warn};
use uuid::Uuid;
use ve_shared::proto::WsEnvelope;

use crate::config::Config;
use crate::credentials::Credentials;
use crate::error::DaemonError;
use crate::Result;

/// Pairing state machine
pub struct Pairing {
    config: Arc<Config>,
    http_client: Client,
}

impl Pairing {
    /// Create a new pairing instance
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            http_client: Client::new(),
        }
    }

    /// Start the pairing process
    ///
    /// This will:
    /// 1. Call daemon-hello to register and get a pair code
    /// 2. Display the pair code to the user
    /// 3. Wait for pairing to complete (polling or WebSocket)
    /// 4. Return the credentials
    pub async fn start(
        &self,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<Credentials> {
        info!("Starting pairing process");

        // 1. Call daemon-hello to register and get pair code
        let hello_url = format!(
            "{}/api/auth/daemon-hello",
            self.config.server_url.trim_end_matches('/')
        );

        // Generate a local pair code (server can override)
        let local_pair_code = generate_pair_code();

        let response = self
            .http_client
            .post(&hello_url)
            .json(&serde_json::json!({
                "pair_code": local_pair_code,
                "host_name": self.config.host_name,
                "platform": self.config.platform,
            }))
            .send()
            .await
            .map_err(|e| DaemonError::PairingFailed {
                reason: format!("HTTP request failed: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(DaemonError::PairingFailed {
                reason: format!("Server returned {}: {}", status, body),
            });
        }

        let json: serde_json::Value = response.json().await.map_err(|e| {
            DaemonError::PairingFailed {
                reason: format!("Failed to parse response: {}", e),
            }
        })?;

        let host_id = json
            .get("host_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::PairingFailed {
                reason: "Missing host_id in response".to_string(),
            })?;

        let host_id = Uuid::parse_str(host_id).map_err(|_| DaemonError::TokenParse)?;

        // The server may generate its own pair code, or use ours
        let pair_code = json
            .get("pair_code")
            .and_then(|v| v.as_str())
            .unwrap_or(&local_pair_code);

        let qr_payload = json
            .get("qr_payload")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // 2. Display pair code to user
        info!(
            pair_code = %pair_code,
            qr_payload = ?qr_payload,
            "=== PAIRING REQUIRED ==="
        );
        eprintln!();
        eprintln!("========================================");
        eprintln!("  Pair this daemon using code:");
        eprintln!("  {}", pair_code);
        eprintln!("========================================");
        if let Some(ref qr) = qr_payload {
            eprintln!("  Or scan: {}", qr);
        }
        eprintln!();

        // 3. Wait for pairing completion
        self.wait_for_pairing(host_id, pair_code, shutdown_rx).await
    }

    /// Wait for pairing to complete
    ///
    /// Uses WebSocket to wait for the 'paired' message.
    /// Falls back to polling if WebSocket authentication is not supported.
    async fn wait_for_pairing(
        &self,
        host_id: Uuid,
        pair_code: &str,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<Credentials> {
        // Try WebSocket with pair_code authentication first
        // Note: This requires server support for ?pair_code= auth
        // If not supported, we'll fall back to polling
        match self
            .wait_via_websocket(pair_code, &mut shutdown_rx)
            .await
        {
            Ok(creds) => return Ok(creds),
            Err(DaemonError::WsConnect(_)) => {
                warn!("WebSocket authentication with pair_code not supported, falling back to polling");
            }
            Err(e) => {
                warn!(error = %e, "WebSocket pairing failed, falling back to polling");
            }
        }

        // Fallback: Polling for pairing status
        self.poll_for_credentials(host_id, shutdown_rx).await
    }

    /// Wait for pairing via WebSocket (preferred method)
    ///
    /// Connects to WebSocket with pair_code as authentication.
    /// Requires server support for ?pair_code= parameter.
    async fn wait_via_websocket(
        &self,
        pair_code: &str,
        shutdown_rx: &mut broadcast::Receiver<()>,
    ) -> Result<Credentials> {
        use futures_util::StreamExt;
        use tokio_tungstenite::connect_async;

        let ws_url = format!(
            "{}/ws/daemon?pair_code={}",
            self.config.server_url.trim_end_matches('/'),
            pair_code
        );

        info!(url = %ws_url, "Attempting WebSocket pairing...");

        let (ws_stream, _) = tokio::select! {
            result = connect_async(&ws_url) => {
                result.map_err(|e| DaemonError::WsConnect(Box::new(e)))?
            }
            _ = shutdown_rx.recv() => {
                return Err(DaemonError::PairingFailed {
                    reason: "Shutdown requested".to_string(),
                });
            }
        };

        let (_, mut receiver) = ws_stream.split();

        // Wait for paired message with timeout
        let timeout = Duration::from_secs(300); // 5 minutes

        let result = tokio::select! {
            result = self.receive_paired_message(&mut receiver) => result,
            _ = tokio::time::sleep(timeout) => {
                Err(DaemonError::PairingTimeout)
            }
            _ = shutdown_rx.recv() => {
                Err(DaemonError::PairingFailed {
                    reason: "Shutdown requested".to_string(),
                })
            }
        };

        result
    }

    /// Receive the paired message from WebSocket
    async fn receive_paired_message(
        &self,
        receiver: &mut futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
    ) -> Result<Credentials> {
        use futures_util::StreamExt;
        use tokio_tungstenite::tungstenite::Message;

        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let envelope: WsEnvelope =
                        serde_json::from_str(&text).map_err(DaemonError::WsMessageParse)?;

                    if envelope.r#type == "paired" {
                        return self.extract_credentials(&envelope);
                    }
                }
                Ok(Message::Close(_)) => {
                    return Err(DaemonError::PairingFailed {
                        reason: "Connection closed before pairing completed".to_string(),
                    });
                }
                _ => {}
            }
        }

        Err(DaemonError::PairingFailed {
            reason: "WebSocket stream ended".to_string(),
        })
    }

    /// Extract credentials from paired message
    fn extract_credentials(&self, envelope: &WsEnvelope) -> Result<Credentials> {
        let host_id = envelope
            .payload
            .get("host_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::PairingFailed {
                reason: "Missing host_id in paired message".to_string(),
            })?;

        let daemon_token = envelope
            .payload
            .get("daemon_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::PairingFailed {
                reason: "Missing daemon_token in paired message".to_string(),
            })?;

        Ok(Credentials::new(
            host_id.to_string(),
            daemon_token.to_string(),
            self.config.server_url.clone(),
        ))
    }

    /// Poll for credentials (fallback method)
    ///
    /// Polls the server periodically to check if pairing has completed.
    /// This requires a server endpoint that returns the token after pairing.
    ///
    /// NOTE: This method requires server support (GET /api/auth/pairing-status).
    /// Currently not implemented - WebSocket pairing is the only supported method.
    async fn poll_for_credentials(
        &self,
        _host_id: Uuid,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<Credentials> {
        // Server does not yet provide a polling endpoint for pairing status.
        // We wait here with a timeout, checking periodically for shutdown.
        warn!(
            "Polling-based pairing is not supported. \
             Server needs to provide GET /api/auth/pairing-status endpoint."
        );

        let timeout = Duration::from_secs(300); // 5 minutes
        let check_interval = Duration::from_secs(30);
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(DaemonError::PairingFailed {
                    reason: "Polling-based pairing not supported and pairing timed out. \
                             Please ensure WebSocket connection is available."
                        .to_string(),
                });
            }

            tokio::select! {
                _ = shutdown_rx.recv() => {
                    return Err(DaemonError::PairingFailed {
                        reason: "Shutdown requested".to_string(),
                    });
                }
                _ = tokio::time::sleep(check_interval) => {
                    // Server endpoint not yet available, continue waiting
                    info!(
                        elapsed_seconds = start.elapsed().as_secs(),
                        "Waiting for pairing... (polling not implemented)"
                    );
                }
            }
        }
    }

    /// Save credentials to disk
    pub fn save_credentials(&self, creds: &Credentials) -> Result<()> {
        let path = self.config.credentials_path();
        creds.save(&path)?;
        info!("Credentials saved to {}", path.display());
        Ok(())
    }
}

/// Generate a random 6-character pair code
///
/// Uses the same charset as the server: uppercase letters and digits,
/// excluding ambiguous characters (I, O, 0, 1).
fn generate_pair_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::rng();

    (0..6)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pair_code() {
        let code = generate_pair_code();

        // Should be 6 characters
        assert_eq!(code.len(), 6);

        // Should only contain valid characters
        for c in code.chars() {
            assert!(CHARSET.contains(&(c as u8)));
        }
    }

    #[test]
    fn test_generate_pair_code_uniqueness() {
        let codes: Vec<_> = (0..100).map(|_| generate_pair_code()).collect();

        // All codes should be unique (with high probability)
        let unique: std::collections::HashSet<_> = codes.iter().collect();
        assert!(unique.len() > 95); // Allow some collisions
    }

    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
}
