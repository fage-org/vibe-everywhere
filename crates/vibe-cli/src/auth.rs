use std::time::{Duration, Instant};

use qrcode::{QrCode, render::unicode};
use reqwest::Client;
use serde::{Deserialize, de::DeserializeOwned};
use thiserror::Error;
use vibe_agent::encryption::{
    EncryptionError, decode_base64, decrypt_box_bundle, encode_base64, encode_base64url,
    random_bytes,
};

use crate::{
    config::Config,
    credentials::{
        Credentials, CredentialsError, clear_credentials, read_credentials,
        write_credentials_data_key, write_credentials_legacy,
    },
};

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const AUTH_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone)]
pub struct PendingTerminalAuth {
    pub public_key: [u8; 32],
    pub secret_key: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct AuthorizedTerminalAuth {
    pub token: String,
    pub response: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "state")]
pub enum AuthRequestResponse {
    #[serde(rename = "requested")]
    Requested,
    #[serde(rename = "authorized")]
    Authorized { token: String, response: String },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthRequestStatusResponse {
    pub status: String,
    pub supports_v2: bool,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Failed to initiate auth: {0}")]
    Request(String),
    #[error("Auth polling failed: {0}")]
    Poll(String),
    #[error("Failed to decrypt auth response")]
    DecryptResponse,
    #[error("Authentication timed out. Please try again.")]
    Timeout,
    #[error("Malformed auth response bundle")]
    MalformedResponseBundle,
    #[error(transparent)]
    Credentials(#[from] CredentialsError),
    #[error(transparent)]
    Encryption(#[from] EncryptionError),
}

impl PendingTerminalAuth {
    pub fn new() -> Self {
        let secret_key = random_bytes::<32>();
        let mut public_key = [0u8; 32];
        dryoc::classic::crypto_core::crypto_scalarmult_base(&mut public_key, &secret_key);

        Self {
            public_key,
            secret_key,
        }
    }

    pub fn public_key_base64(&self) -> String {
        encode_base64(&self.public_key)
    }

    pub fn deep_link_url(&self) -> String {
        format!("vibe:///terminal?{}", encode_base64url(&self.public_key))
    }

    pub fn web_auth_url(&self, config: &Config) -> String {
        format!(
            "{}/terminal/connect#key={}",
            config.webapp_url,
            encode_base64url(&self.public_key)
        )
    }
}

impl Default for PendingTerminalAuth {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_auth_qr(pending: &PendingTerminalAuth) -> Result<String, AuthError> {
    let code = QrCode::new(pending.deep_link_url().as_bytes())
        .map_err(|error: qrcode::types::QrError| AuthError::Request(error.to_string()))?;
    Ok(code.render::<unicode::Dense1x2>().quiet_zone(false).build())
}

pub fn auth_status(config: &Config) -> Option<Credentials> {
    read_credentials(config)
}

pub fn auth_logout(config: &Config) -> Result<(), AuthError> {
    clear_credentials(config)?;
    Ok(())
}

pub async fn request_terminal_auth(
    client: &Client,
    config: &Config,
    pending: &PendingTerminalAuth,
) -> Result<AuthRequestResponse, AuthError> {
    post_auth_request(client, config, pending, "initiate").await
}

pub async fn poll_until_authorized(
    client: &Client,
    config: &Config,
    pending: &PendingTerminalAuth,
) -> Result<AuthorizedTerminalAuth, AuthError> {
    poll_until_authorized_with_timing(client, config, pending, POLL_INTERVAL, AUTH_TIMEOUT).await
}

pub async fn auth_request_status(
    client: &Client,
    config: &Config,
    pending: &PendingTerminalAuth,
) -> Result<AuthRequestStatusResponse, AuthError> {
    let public_key = url::form_urlencoded::byte_serialize(pending.public_key_base64().as_bytes())
        .collect::<String>();
    let url = format!(
        "{}/v1/auth/request/status?publicKey={}",
        config.server_url, public_key
    );
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| AuthError::Poll(error.to_string()))?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(AuthError::Poll(format!("{status}: {body}")));
    }
    serde_json::from_str(&body).map_err(|error| AuthError::Poll(error.to_string()))
}

pub fn complete_terminal_auth(
    config: &Config,
    pending: &PendingTerminalAuth,
    authorized: AuthorizedTerminalAuth,
) -> Result<Credentials, AuthError> {
    let encrypted = decode_base64(&authorized.response)?;
    let decrypted =
        decrypt_box_bundle(&encrypted, &pending.secret_key).ok_or(AuthError::DecryptResponse)?;
    match decrypted.as_slice() {
        bytes if bytes.len() == 32 => {
            let secret: [u8; 32] = bytes
                .try_into()
                .map_err(|_| AuthError::MalformedResponseBundle)?;
            write_credentials_legacy(config, authorized.token.clone(), secret)?;
            Ok(read_credentials(config).expect("credentials should be readable after write"))
        }
        bytes if bytes.len() == 33 && bytes[0] == 0 => {
            let public_key: [u8; 32] = bytes[1..33]
                .try_into()
                .map_err(|_| AuthError::MalformedResponseBundle)?;
            let machine_key = random_bytes::<32>();
            write_credentials_data_key(config, authorized.token.clone(), public_key, machine_key)?;
            Ok(read_credentials(config).expect("credentials should be readable after write"))
        }
        _ => Err(AuthError::MalformedResponseBundle),
    }
}

async fn poll_until_authorized_with_timing(
    client: &Client,
    config: &Config,
    pending: &PendingTerminalAuth,
    poll_interval: Duration,
    auth_timeout: Duration,
) -> Result<AuthorizedTerminalAuth, AuthError> {
    let started_at = Instant::now();
    loop {
        if started_at.elapsed() >= auth_timeout {
            return Err(AuthError::Timeout);
        }

        tokio::time::sleep(poll_interval).await;
        match post_auth_request(client, config, pending, "poll").await? {
            AuthRequestResponse::Requested => {}
            AuthRequestResponse::Authorized { token, response } => {
                return Ok(AuthorizedTerminalAuth { token, response });
            }
        }
    }
}

async fn post_auth_request<T: DeserializeOwned>(
    client: &Client,
    config: &Config,
    pending: &PendingTerminalAuth,
    mode: &str,
) -> Result<T, AuthError> {
    let response = client
        .post(format!("{}/v1/auth/request", config.server_url))
        .json(&serde_json::json!({
            "publicKey": pending.public_key_base64(),
            "supportsV2": true,
        }))
        .send()
        .await
        .map_err(|error| match mode {
            "poll" => AuthError::Poll(error.to_string()),
            _ => AuthError::Request(error.to_string()),
        })?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        let message = if body.is_empty() {
            status.to_string()
        } else {
            format!("{status}: {body}")
        };
        return Err(match mode {
            "poll" => AuthError::Poll(message),
            _ => AuthError::Request(message),
        });
    }

    serde_json::from_str(&body).map_err(|error| match mode {
        "poll" => AuthError::Poll(error.to_string()),
        _ => AuthError::Request(error.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use std::{
        net::TcpListener as StdTcpListener,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::Duration,
    };

    use axum::{
        Json, Router,
        extract::State,
        routing::{get, post},
    };
    use reqwest::Client;
    use serde_json::{Value, json};
    use tokio::{sync::oneshot, task::JoinHandle};
    use vibe_agent::encryption::{
        decode_base64url, libsodium_encrypt_for_public_key, random_bytes,
    };

    use crate::config::Config;

    use super::{
        AuthRequestResponse, PendingTerminalAuth, auth_logout, auth_request_status, auth_status,
        complete_terminal_auth, poll_until_authorized_with_timing, request_terminal_auth,
    };

    struct MockAuthServer {
        server_url: String,
        shutdown: Option<oneshot::Sender<()>>,
        task: Option<JoinHandle<()>>,
    }

    impl MockAuthServer {
        async fn start(router: Router) -> Self {
            let std_listener = StdTcpListener::bind("127.0.0.1:0").unwrap();
            let addr = std_listener.local_addr().unwrap();
            std_listener.set_nonblocking(true).unwrap();
            let listener = tokio::net::TcpListener::from_std(std_listener).unwrap();
            let (shutdown_tx, shutdown_rx) = oneshot::channel();
            let task = tokio::spawn(async move {
                axum::serve(listener, router)
                    .with_graceful_shutdown(async {
                        let _ = shutdown_rx.await;
                    })
                    .await
                    .unwrap();
            });

            Self {
                server_url: format!("http://{addr}"),
                shutdown: Some(shutdown_tx),
                task: Some(task),
            }
        }

        fn config(&self) -> (tempfile::TempDir, Config) {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let config = Config::from_sources(
                Some(self.server_url.clone()),
                Some("https://app.vibe.engineering".into()),
                Some(temp_dir.path().as_os_str().to_owned()),
                None,
            )
            .unwrap();
            (temp_dir, config)
        }
    }

    impl Drop for MockAuthServer {
        fn drop(&mut self) {
            if let Some(sender) = self.shutdown.take() {
                let _ = sender.send(());
            }
            if let Some(task) = self.task.take() {
                task.abort();
            }
        }
    }

    #[test]
    fn pending_terminal_auth_uses_vibe_terminal_deep_link() {
        let pending = PendingTerminalAuth::new();

        let qr_url = pending.deep_link_url();
        assert!(qr_url.starts_with("vibe:///terminal?"));
        let encoded = qr_url.trim_start_matches("vibe:///terminal?");
        let decoded = decode_base64url(encoded).unwrap();
        assert_eq!(decoded, pending.public_key);
    }

    #[test]
    fn terminal_auth_generates_web_connect_url() {
        let pending = PendingTerminalAuth::new();
        let config = Config::from_sources(
            Some("http://127.0.0.1:3005".into()),
            Some("https://app.vibe.example".into()),
            Some("/tmp/vibe-cli-auth".into()),
            None,
        )
        .unwrap();

        assert!(
            pending
                .web_auth_url(&config)
                .starts_with("https://app.vibe.example/terminal/connect#key=")
        );
    }

    #[test]
    fn complete_terminal_auth_decrypts_legacy_secret() {
        let pending = PendingTerminalAuth::new();
        let secret = random_bytes::<32>();
        let response = libsodium_encrypt_for_public_key(&secret, &pending.public_key);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config = crate::config::Config::from_sources(
            Some("http://127.0.0.1:3005".into()),
            Some("https://app.vibe.engineering".into()),
            Some(temp_dir.path().as_os_str().to_owned()),
            None,
        )
        .unwrap();

        let credentials = complete_terminal_auth(
            &config,
            &pending,
            super::AuthorizedTerminalAuth {
                token: "token-1".into(),
                response: vibe_agent::encryption::encode_base64(&response),
            },
        )
        .unwrap();

        assert!(matches!(
            credentials.encryption,
            crate::credentials::CredentialEncryption::Legacy { secret: value, .. } if value == secret
        ));
    }

    #[test]
    fn complete_terminal_auth_decrypts_v2_public_key_bundle() {
        let pending = PendingTerminalAuth::new();
        let public_key = random_bytes::<32>();
        let mut response_bundle = vec![0u8];
        response_bundle.extend_from_slice(&public_key);
        let response = libsodium_encrypt_for_public_key(&response_bundle, &pending.public_key);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config = crate::config::Config::from_sources(
            Some("http://127.0.0.1:3005".into()),
            Some("https://app.vibe.engineering".into()),
            Some(temp_dir.path().as_os_str().to_owned()),
            None,
        )
        .unwrap();

        let credentials = complete_terminal_auth(
            &config,
            &pending,
            super::AuthorizedTerminalAuth {
                token: "token-2".into(),
                response: vibe_agent::encryption::encode_base64(&response),
            },
        )
        .unwrap();

        assert!(matches!(
            credentials.encryption,
            crate::credentials::CredentialEncryption::DataKey { public_key: pk, .. } if pk == public_key
        ));
    }

    #[tokio::test]
    async fn request_terminal_auth_posts_public_key_and_supports_v2() {
        #[derive(Clone)]
        struct RequestState {
            captured_public_key: Arc<std::sync::Mutex<Option<String>>>,
            captured_supports_v2: Arc<std::sync::Mutex<Option<bool>>>,
        }

        let state = RequestState {
            captured_public_key: Arc::new(std::sync::Mutex::new(None)),
            captured_supports_v2: Arc::new(std::sync::Mutex::new(None)),
        };
        let server = MockAuthServer::start(
            Router::new()
                .route(
                    "/v1/auth/request",
                    post(
                        |State(state): State<RequestState>, Json(body): Json<Value>| async move {
                            *state.captured_public_key.lock().unwrap() = body
                                .get("publicKey")
                                .and_then(Value::as_str)
                                .map(ToOwned::to_owned);
                            *state.captured_supports_v2.lock().unwrap() =
                                body.get("supportsV2").and_then(Value::as_bool);
                            Json(json!({ "state": "requested" }))
                        },
                    ),
                )
                .with_state(state.clone()),
        )
        .await;
        let (_temp_dir, config) = server.config();
        let client = Client::new();
        let pending = PendingTerminalAuth::new();

        let response = request_terminal_auth(&client, &config, &pending)
            .await
            .unwrap();
        assert!(matches!(response, AuthRequestResponse::Requested));
        assert_eq!(
            state.captured_public_key.lock().unwrap().as_deref(),
            Some(pending.public_key_base64().as_str())
        );
        assert_eq!(*state.captured_supports_v2.lock().unwrap(), Some(true));
    }

    #[tokio::test]
    async fn poll_until_authorized_succeeds_after_requested_response() {
        #[derive(Clone)]
        struct PollState {
            calls: Arc<AtomicUsize>,
            response: String,
        }

        let pending = PendingTerminalAuth::new();
        let secret = random_bytes::<32>();
        let response = vibe_agent::encryption::encode_base64(&libsodium_encrypt_for_public_key(
            &secret,
            &pending.public_key,
        ));
        let state = PollState {
            calls: Arc::new(AtomicUsize::new(0)),
            response,
        };
        let server = MockAuthServer::start(
            Router::new()
                .route(
                    "/v1/auth/request",
                    post(|State(state): State<PollState>| async move {
                        let call = state.calls.fetch_add(1, Ordering::SeqCst);
                        if call == 0 {
                            Json(json!({ "state": "requested" }))
                        } else {
                            Json(json!({
                                "state": "authorized",
                                "token": "token-1",
                                "response": state.response,
                            }))
                        }
                    }),
                )
                .with_state(state.clone()),
        )
        .await;
        let (_temp_dir, config) = server.config();
        let client = Client::new();

        let authorized = poll_until_authorized_with_timing(
            &client,
            &config,
            &pending,
            Duration::from_millis(10),
            Duration::from_secs(1),
        )
        .await
        .unwrap();

        assert_eq!(authorized.token, "token-1");
        assert_eq!(state.calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn auth_request_status_reads_supports_v2_state() {
        let pending = PendingTerminalAuth::new();
        let server = MockAuthServer::start(Router::new().route(
            "/v1/auth/request/status",
            get(|| async { Json(json!({"status":"pending","supportsV2":true})) }),
        ))
        .await;
        let (_temp_dir, config) = server.config();
        let client = Client::new();
        let status = auth_request_status(&client, &config, &pending)
            .await
            .unwrap();
        assert_eq!(status.status, "pending");
        assert!(status.supports_v2);
    }

    #[tokio::test]
    async fn poll_until_authorized_times_out() {
        let pending = PendingTerminalAuth::new();
        let server = MockAuthServer::start(Router::new().route(
            "/v1/auth/request",
            post(|| async { Json(json!({ "state": "requested" })) }),
        ))
        .await;
        let (_temp_dir, config) = server.config();
        let client = Client::new();

        let error = poll_until_authorized_with_timing(
            &client,
            &config,
            &pending,
            Duration::from_millis(10),
            Duration::from_millis(30),
        )
        .await
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "Authentication timed out. Please try again."
        );
    }

    #[test]
    fn logout_clears_credentials_and_status_reports_none() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config = Config::from_sources(
            Some("http://127.0.0.1:3005".into()),
            Some("https://app.vibe.engineering".into()),
            Some(temp_dir.path().as_os_str().to_owned()),
            None,
        )
        .unwrap();
        let secret = [7u8; 32];

        crate::credentials::write_credentials(&config, "token-1", secret).unwrap();
        assert!(auth_status(&config).is_some());
        auth_logout(&config).unwrap();
        assert!(auth_status(&config).is_none());
    }
}
