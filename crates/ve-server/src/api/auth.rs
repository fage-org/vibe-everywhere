//! Authentication API Handlers
//!
//! Device registration, daemon hello, and pairing endpoints.

use axum::{extract::State, Json};
use rand::Rng;
use std::sync::Arc;
use uuid::Uuid;

use ve_shared::jwt::JwtManager;
use ve_shared::models::{RegisterDeviceRequest, RegisterDeviceResponse};

use crate::error::{Result, ServerError};
use crate::state::AppState;
use crate::validation::{validate_device_name, validate_host_name};

/// POST /api/auth/register-device
///
/// Register a new client device and receive a JWT token.
pub async fn register_device(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterDeviceRequest>,
) -> Result<Json<RegisterDeviceResponse>> {
    // Validate device name
    validate_device_name(&req.device_name)?;

    let device_id = Uuid::new_v4();
    let device_id_str = device_id.to_string();

    // Create JWT manager
    let jwt_manager = JwtManager::new(&state.config.jwt_secret, state.config.jwt_expiration());

    // Generate token
    let token = jwt_manager.create_client_token(device_id, &req.device_name)?;

    // Store device in database
    let device_type_str = match req.device_type {
        ve_shared::types::DeviceType::Mobile => "mobile",
        ve_shared::types::DeviceType::Desktop => "desktop",
    };

    sqlx::query!(
        r#"
        INSERT INTO client_devices (device_id, device_name, device_type, server_url)
        VALUES (?, ?, ?, ?)
        "#,
        device_id_str,
        req.device_name,
        device_type_str,
        req.server_url,
    )
    .execute(&state.db)
    .await?;

    tracing::info!(%device_id, "Device registered");

    Ok(Json(RegisterDeviceResponse { device_id, token }))
}

/// Daemon hello request
#[derive(Debug, serde::Deserialize)]
pub struct DaemonHelloRequest {
    pub pair_code: String,
    pub host_name: String,
    pub platform: String,
}

/// Daemon hello response
#[derive(Debug, serde::Serialize)]
pub struct DaemonHelloResponse {
    pub host_id: Uuid,
    pub status: String,
}

/// POST /api/auth/daemon-hello
///
/// Daemon initiates pairing by providing a pair code and host info.
/// This creates a pending pairing context.
pub async fn daemon_hello(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DaemonHelloRequest>,
) -> Result<Json<DaemonHelloResponse>> {
    // Validate host name
    validate_host_name(&req.host_name)?;

    // Validate platform is one of the allowed values
    if !matches!(req.platform.as_str(), "linux" | "macos" | "wsl") {
        return Err(ServerError::BadRequest(format!(
            "Invalid platform: '{}'. Must be one of: linux, macos, wsl",
            req.platform
        )));
    }

    // Generate host ID
    let host_id = Uuid::new_v4();
    let host_id_str = host_id.to_string();

    // Generate a new pair code if not provided (daemon generates its own)
    let pair_code = if req.pair_code.is_empty() {
        generate_pair_code()
    } else {
        req.pair_code
    };

    // Calculate expiration time
    let expires_at = chrono::Utc::now() + state.config.pair_code_ttl();
    let expires_at_str = expires_at.to_rfc3339();

    // Generate QR payload
    let qr_payload = format!("vibe://pair/{}", pair_code);

    // Store pending pairing
    sqlx::query!(
        r#"
        INSERT INTO pairing_codes (pair_code, host_id, host_name, platform, qr_payload, expires_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        pair_code,
        host_id_str,
        req.host_name,
        req.platform,
        qr_payload,
        expires_at_str,
    )
    .execute(&state.db)
    .await?;

    // Create host record in pending state
    sqlx::query!(
        r#"
        INSERT INTO hosts (host_id, host_name, platform, pair_status, pair_code, qr_payload)
        VALUES (?, ?, ?, 'pending', ?, ?)
        "#,
        host_id_str,
        req.host_name,
        req.platform,
        pair_code,
        qr_payload,
    )
    .execute(&state.db)
    .await?;

    tracing::info!(%host_id, "Daemon hello received");
    tracing::debug!(%host_id, %pair_code, "Pair code generated");

    Ok(Json(DaemonHelloResponse {
        host_id,
        status: "pending".to_string(),
    }))
}

/// Pair request from client
#[derive(Debug, serde::Deserialize)]
pub struct PairRequest {
    pub pair_code: String,
}

/// Pair response
#[derive(Debug, serde::Serialize)]
pub struct PairResponse {
    pub host_id: Uuid,
    pub host_name: String,
}

/// POST /api/auth/pair
///
/// Client completes pairing by providing the pair code.
/// This validates the code and establishes the daemon binding.
pub async fn pair(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PairRequest>,
) -> Result<Json<PairResponse>> {
    let pair_code = req.pair_code.clone();

    // Look up the pairing code
    let pairing = sqlx::query!(
        r#"
        SELECT pair_code, host_id, host_name, used, expires_at
        FROM pairing_codes
        WHERE pair_code = ?
        "#,
        pair_code,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::PairCodeExpired)?;

    // Check if already used
    if pairing.used != 0 {
        return Err(ServerError::PairCodeUsed);
    }

    // Check expiration
    let expires_at = chrono::DateTime::parse_from_rfc3339(&pairing.expires_at)
        .map_err(|e| ServerError::Internal(format!("Invalid expiration time: {}", e)))?;

    if chrono::Utc::now() > expires_at {
        return Err(ServerError::PairCodeExpired);
    }

    let host_id = Uuid::parse_str(&pairing.host_id)
        .map_err(|e| ServerError::Internal(format!("Invalid host ID: {}", e)))?;

    let host_id_str = pairing.host_id.clone();

    // Mark pairing code as used
    sqlx::query!(
        r#"
        UPDATE pairing_codes SET used = 1 WHERE pair_code = ?
        "#,
        pair_code,
    )
    .execute(&state.db)
    .await?;

    // Update host status to paired
    sqlx::query!(
        r#"
        UPDATE hosts
        SET pair_status = 'paired', pair_code = NULL, qr_payload = NULL, updated_at = datetime('now')
        WHERE host_id = ?
        "#,
        host_id_str,
    )
    .execute(&state.db)
    .await?;

    // Generate daemon JWT token
    let jwt_manager = JwtManager::new(&state.config.jwt_secret, state.config.jwt_expiration());
    let daemon_token = jwt_manager.create_daemon_token(host_id, &pairing.host_name)?;

    // Notify daemon via WebSocket if connected
    let _ = state.hub.send_to_daemon(
        &host_id,
        ve_shared::proto::DaemonMessage::Paired {
            host_id,
            daemon_token,
        },
    );

    tracing::info!(%host_id, "Pairing completed");

    Ok(Json(PairResponse {
        host_id,
        host_name: pairing.host_name,
    }))
}

/// Generate a random 6-character pair code
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
