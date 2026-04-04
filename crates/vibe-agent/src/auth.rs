use std::time::{Duration, Instant};

use qrcode::{QrCode, render::unicode};
use reqwest::Client;
use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::{
    config::Config,
    credentials::{
        Credentials, CredentialsError, clear_credentials, read_credentials, write_credentials,
    },
    encryption::{
        EncryptionError, decode_base64, decrypt_box_bundle, derive_content_key_pair, encode_base64,
        encode_base64url, random_bytes,
    },
};

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const AUTH_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone)]
pub struct PendingAccountLink {
    pub public_key: [u8; 32],
    pub secret_key: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct AuthorizedAccountLink {
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

impl PendingAccountLink {
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

    pub fn qr_url(&self) -> String {
        format!("vibe:///account?{}", encode_base64url(&self.public_key))
    }
}

impl Default for PendingAccountLink {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_auth_qr(pending: &PendingAccountLink) -> Result<String, AuthError> {
    let code = QrCode::new(pending.qr_url().as_bytes())
        .map_err(|error| AuthError::Request(error.to_string()))?;
    Ok(code.render::<unicode::Dense1x2>().quiet_zone(false).build())
}

pub fn auth_status(config: &Config) -> Option<Credentials> {
    read_credentials(config)
}

pub fn auth_logout(config: &Config) -> Result<(), AuthError> {
    clear_credentials(config)?;
    Ok(())
}

pub async fn request_account_link(
    client: &Client,
    config: &Config,
    pending: &PendingAccountLink,
) -> Result<AuthRequestResponse, AuthError> {
    post_auth_request(
        client,
        config,
        pending,
        "/v1/auth/account/request",
        "initiate",
    )
    .await
}

pub async fn poll_until_authorized(
    client: &Client,
    config: &Config,
    pending: &PendingAccountLink,
) -> Result<AuthorizedAccountLink, AuthError> {
    poll_until_authorized_with_timing(client, config, pending, POLL_INTERVAL, AUTH_TIMEOUT).await
}

async fn poll_until_authorized_with_timing(
    client: &Client,
    config: &Config,
    pending: &PendingAccountLink,
    poll_interval: Duration,
    auth_timeout: Duration,
) -> Result<AuthorizedAccountLink, AuthError> {
    let started_at = Instant::now();
    loop {
        if started_at.elapsed() >= auth_timeout {
            return Err(AuthError::Timeout);
        }

        tokio::time::sleep(poll_interval).await;
        match post_auth_request(client, config, pending, "/v1/auth/account/request", "poll").await?
        {
            AuthRequestResponse::Requested => {}
            AuthRequestResponse::Authorized { token, response } => {
                return Ok(AuthorizedAccountLink { token, response });
            }
        }
    }
}

pub fn complete_account_link(
    config: &Config,
    pending: &PendingAccountLink,
    authorized: AuthorizedAccountLink,
) -> Result<Credentials, AuthError> {
    let encrypted = decode_base64(&authorized.response)?;
    let secret =
        decrypt_box_bundle(&encrypted, &pending.secret_key).ok_or(AuthError::DecryptResponse)?;
    let secret: [u8; 32] = secret
        .as_slice()
        .try_into()
        .map_err(|_| AuthError::MalformedResponseBundle)?;
    write_credentials(config, authorized.token.clone(), secret)?;

    Ok(Credentials {
        token: authorized.token,
        secret,
        content_key_pair: derive_content_key_pair(&secret),
    })
}

async fn post_auth_request<T: DeserializeOwned>(
    client: &Client,
    config: &Config,
    pending: &PendingAccountLink,
    path: &str,
    mode: &str,
) -> Result<T, AuthError> {
    let response = client
        .post(format!("{}{}", config.server_url, path))
        .json(&serde_json::json!({
            "publicKey": pending.public_key_base64(),
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

    use axum::{Json, Router, extract::State, routing::post};
    use reqwest::Client;
    use serde_json::{Value, json};
    use tokio::{sync::oneshot, task::JoinHandle};

    use crate::{
        config::Config,
        credentials::write_credentials,
        encryption::{decode_base64url, libsodium_encrypt_for_public_key, random_bytes},
    };

    use super::{
        AuthRequestResponse, PendingAccountLink, auth_logout, auth_status, complete_account_link,
        poll_until_authorized_with_timing, request_account_link,
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
                Some(temp_dir.path().as_os_str().to_owned()),
            )
            .unwrap();
            (temp_dir, config)
        }

        async fn shutdown(mut self) {
            if let Some(sender) = self.shutdown.take() {
                let _ = sender.send(());
            }
            if let Some(task) = self.task.take() {
                let _ = task.await;
            }
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
    fn pending_account_link_uses_vibe_deep_link() {
        let pending = PendingAccountLink::new();

        let qr_url = pending.qr_url();
        assert!(qr_url.starts_with("vibe:///account?"));
        let encoded = qr_url.trim_start_matches("vibe:///account?");
        let decoded = decode_base64url(encoded).unwrap();
        assert_eq!(decoded, pending.public_key);
    }

    #[test]
    fn complete_account_link_decrypts_and_builds_credentials() {
        let pending = PendingAccountLink::new();
        let secret = random_bytes::<32>();
        let response = libsodium_encrypt_for_public_key(&secret, &pending.public_key);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config = crate::config::Config::from_sources(
            Some("http://127.0.0.1:3005".into()),
            Some(temp_dir.path().as_os_str().to_owned()),
        )
        .unwrap();

        let credentials = complete_account_link(
            &config,
            &pending,
            super::AuthorizedAccountLink {
                token: "token-1".into(),
                response: crate::encryption::encode_base64(&response),
            },
        )
        .unwrap();

        assert_eq!(credentials.token, "token-1");
        assert_eq!(credentials.secret, secret);
    }

    #[tokio::test]
    async fn request_account_link_posts_public_key_and_reads_requested_state() {
        #[derive(Clone)]
        struct RequestState {
            captured_public_key: Arc<std::sync::Mutex<Option<String>>>,
        }

        let state = RequestState {
            captured_public_key: Arc::new(std::sync::Mutex::new(None)),
        };
        let server = MockAuthServer::start(
            Router::new()
                .route(
                    "/v1/auth/account/request",
                    post(
                        |State(state): State<RequestState>, Json(body): Json<Value>| async move {
                            *state.captured_public_key.lock().unwrap() = body
                                .get("publicKey")
                                .and_then(Value::as_str)
                                .map(ToOwned::to_owned);
                            Json(json!({ "state": "requested" }))
                        },
                    ),
                )
                .with_state(state.clone()),
        )
        .await;
        let (_temp_dir, config) = server.config();
        let pending = PendingAccountLink::new();

        let response = request_account_link(&Client::new(), &config, &pending)
            .await
            .unwrap();

        assert!(matches!(response, AuthRequestResponse::Requested));
        assert_eq!(
            state.captured_public_key.lock().unwrap().as_deref(),
            Some(pending.public_key_base64().as_str())
        );

        server.shutdown().await;
    }

    #[tokio::test]
    async fn poll_until_authorized_succeeds_after_requested_response() {
        let pending = PendingAccountLink::new();
        let secret = random_bytes::<32>();
        let response = crate::encryption::encode_base64(&libsodium_encrypt_for_public_key(
            &secret,
            &pending.public_key,
        ));
        let call_count = Arc::new(AtomicUsize::new(0));
        let server = MockAuthServer::start(Router::new().route(
            "/v1/auth/account/request",
            post({
                let call_count = call_count.clone();
                move || {
                    let call_count = call_count.clone();
                    let response = response.clone();
                    async move {
                        let count = call_count.fetch_add(1, Ordering::SeqCst);
                        if count == 0 {
                            Json(json!({ "state": "requested" }))
                        } else {
                            Json(json!({
                                "state": "authorized",
                                "token": "token-1",
                                "response": response,
                            }))
                        }
                    }
                }
            }),
        ))
        .await;
        let (_temp_dir, config) = server.config();

        let authorized = poll_until_authorized_with_timing(
            &Client::new(),
            &config,
            &pending,
            Duration::from_millis(10),
            Duration::from_millis(200),
        )
        .await
        .unwrap();

        assert_eq!(authorized.token, "token-1");
        assert!(call_count.load(Ordering::SeqCst) >= 2);

        server.shutdown().await;
    }

    #[tokio::test]
    async fn poll_until_authorized_times_out() {
        let server = MockAuthServer::start(Router::new().route(
            "/v1/auth/account/request",
            post(|| async { Json(json!({ "state": "requested" })) }),
        ))
        .await;
        let (_temp_dir, config) = server.config();
        let pending = PendingAccountLink::new();

        let error = poll_until_authorized_with_timing(
            &Client::new(),
            &config,
            &pending,
            Duration::from_millis(10),
            Duration::from_millis(35),
        )
        .await
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "Authentication timed out. Please try again."
        );

        server.shutdown().await;
    }

    #[test]
    fn logout_clears_credentials_and_status_reports_none() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config = Config::from_sources(
            Some("http://127.0.0.1:3005".into()),
            Some(temp_dir.path().as_os_str().to_owned()),
        )
        .unwrap();
        write_credentials(&config, "token-1", [4u8; 32]).unwrap();

        let before = auth_status(&config);
        assert!(before.is_some());

        auth_logout(&config).unwrap();

        assert!(auth_status(&config).is_none());
    }
}
