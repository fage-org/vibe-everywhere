//! Authentication API Handlers
//!
//! Device registration, daemon hello, and pairing endpoints.

use axum::{
    extract::{ConnectInfo, Extension, Query, State},
    http::HeaderMap,
    Json,
};

use rand::Rng;
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;

use ve_shared::jwt::{Claims, JwtManager};
use ve_shared::models::{PairResponse, RegisterDeviceRequest, RegisterDeviceResponse};
use ve_shared::pairing_proof::PairingProof;

use crate::authz::require_bootstrap_device_id;
use crate::error::{Result, ServerError};
use crate::state::AppState;
use crate::utils;
use crate::validation::{validate_device_name, validate_host_name, validate_pair_code};

/// POST /api/auth/register-device
///
/// Register a new client device and receive a JWT token.
pub async fn register_device(
    State(state): State<Arc<AppState>>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
    Json(req): Json<RegisterDeviceRequest>,
) -> Result<Json<RegisterDeviceResponse>> {
    // Validate device name
    validate_device_name(&req.device_name)?;

    if !state.auth_throttle.allow_register_device(remote_addr.ip()) {
        return Err(ServerError::TooManyRequests(
            "Too many device registration attempts from this source".to_string(),
        ));
    }

    let device_id = Uuid::new_v4();
    let device_id_str = device_id.to_string();

    // Create JWT manager
    let jwt_manager = JwtManager::new(&state.config.jwt_secret, state.config.jwt_expiration());

    // Generate token
    let token = jwt_manager.create_client_bootstrap_token(device_id, &req.device_name)?;

    // Store device in database
    let device_type_str = match req.device_type {
        ve_shared::types::DeviceType::Mobile => "mobile",
        ve_shared::types::DeviceType::Desktop => "desktop",
    };

    sqlx::query(
        r#"
        INSERT INTO client_devices (device_id, device_name, device_type, server_url)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(&device_id_str)
    .bind(&req.device_name)
    .bind(device_type_str)
    .bind(&req.server_url)
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
    pub pairing_proof: PairingProof,
}

/// Daemon hello response
#[derive(Debug, serde::Serialize)]
pub struct DaemonHelloResponse {
    pub host_id: Uuid,
    pub status: String,
    pub pair_code: String,
    pub qr_payload: String,
    pub pairing_secret: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct PairingStatusQuery {
    pub host_id: Uuid,
}

#[derive(Debug, serde::Serialize)]
pub struct PairingStatusResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daemon_token: Option<String>,
}

/// POST /api/auth/daemon-hello
///
/// Daemon initiates pairing by providing a pair code and host info.
/// This creates a pending pairing context.
pub async fn daemon_hello(
    State(state): State<Arc<AppState>>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
    Json(req): Json<DaemonHelloRequest>,
) -> Result<Json<DaemonHelloResponse>> {
    validate_host_name(&req.host_name)?;
    let host_name = req.host_name.trim().to_string();

    if !matches!(req.platform.as_str(), "linux" | "macos" | "windows" | "wsl") {
        return Err(ServerError::Validation(
            crate::validation::ValidationError::InvalidChars { field: "platform" },
        ));
    }

    req.pairing_proof.verify().map_err(|_| {
        ServerError::Validation(crate::validation::ValidationError::InvalidChars {
            field: "pairing_proof",
        })
    })?;

    // Generate a new pair code if not provided (daemon generates its own)
    let pair_code = if req.pair_code.is_empty() {
        generate_pair_code()
    } else {
        req.pair_code.trim().to_ascii_uppercase()
    };
    validate_pair_code(&pair_code)?;

    if !state.auth_throttle.allow_daemon_hello(remote_addr.ip()) {
        return Err(ServerError::TooManyRequests(
            "Too many pairing attempts for this host".to_string(),
        ));
    }

    // Generate host ID
    let host_id = Uuid::new_v4();
    let host_id_str = host_id.to_string();

    // Calculate expiration time
    let expires_at = chrono::Utc::now() + state.config.pair_code_ttl();
    let expires_at_str = expires_at.to_rfc3339();

    // Generate QR payload
    let qr_payload = format!("vibe://pair/{}", pair_code);

    let pairing_secret = generate_pairing_secret();

    let mut tx = state.db.begin().await?;

    // Store pending pairing
    sqlx::query(
        r#"
        INSERT INTO pairing_codes (pair_code, host_id, host_name, platform, qr_payload, pairing_secret, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(&pair_code)
    .bind(&host_id_str)
    .bind(&host_name)
    .bind(&req.platform)
    .bind(&qr_payload)
    .bind(&pairing_secret)
    .bind(&expires_at_str)
    .execute(&mut *tx)
    .await?;

    // Create host record in pending state
    sqlx::query(
        r#"
        INSERT INTO hosts (host_id, host_name, platform, pair_status, pair_code, qr_payload)
        VALUES ($1, $2, $3, 'pending', $4, $5)
        "#,
    )
    .bind(&host_id_str)
    .bind(&host_name)
    .bind(&req.platform)
    .bind(&pair_code)
    .bind(&qr_payload)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    tracing::info!(%host_id, "Daemon hello received");

    Ok(Json(DaemonHelloResponse {
        host_id,
        status: "pending".to_string(),
        pair_code,
        qr_payload,
        pairing_secret,
    }))
}

/// GET /api/auth/pairing-status
pub async fn pairing_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<PairingStatusQuery>,
) -> Result<Json<PairingStatusResponse>> {
    let host_id_str = query.host_id.to_string();
    let pairing_secret = headers
        .get("x-pairing-secret")
        .and_then(|value| value.to_str().ok())
        .ok_or(ServerError::Unauthorized)?;
    let row: (String, String, Option<String>, String, i64) = sqlx::query_as(
        r#"
        SELECT hosts.pair_status, hosts.host_name, pairing_codes.pairing_secret, pairing_codes.expires_at,
               pairing_codes.used
        FROM pairing_codes
        JOIN hosts ON hosts.host_id = pairing_codes.host_id
        WHERE pairing_codes.host_id = $1
        "#,
    )
    .bind(&host_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::PairCodeExpired)?;

    if row.2.as_deref() != Some(pairing_secret) {
        return Err(ServerError::Unauthorized);
    }

    let expires_at = utils::parse_sqlite_timestamp(&row.3)
        .map_err(|e| ServerError::Internal(format!("Invalid expiration time: {}", e)))?
        .with_timezone(&chrono::Utc);
    let now = chrono::Utc::now();

    if now > expires_at {
        return Err(ServerError::PairCodeExpired);
    }

    if row.0 == "paired" {
        let clear_secret = sqlx::query(
            r#"
            UPDATE pairing_codes
            SET pairing_secret = NULL
            WHERE host_id = $1 AND pairing_secret = $2
            "#,
        )
        .bind(&host_id_str)
        .bind(pairing_secret)
        .execute(&state.db)
        .await?;

        if clear_secret.rows_affected() == 0 {
            return Err(ServerError::Unauthorized);
        }

        let jwt_manager = JwtManager::new(&state.config.jwt_secret, state.config.jwt_expiration());
        let daemon_token = jwt_manager.create_daemon_token(query.host_id, &row.1)?;
        return Ok(Json(PairingStatusResponse {
            status: "paired".to_string(),
            daemon_token: Some(daemon_token),
        }));
    }

    if row.4 != 0 {
        return Err(ServerError::PairCodeUsed);
    }

    Ok(Json(PairingStatusResponse {
        status: "pending".to_string(),
        daemon_token: None,
    }))
}

/// Pair request from client
#[derive(Debug, serde::Deserialize)]
pub struct PairRequest {
    pub pair_code: String,
}

/// POST /api/auth/pair
///
/// Bootstrap client completes pairing by providing the pair code.
/// This validates the code, establishes the daemon binding, and returns a formal client token.
pub async fn pair(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<PairRequest>,
) -> Result<Json<PairResponse>> {
    let device_id = require_bootstrap_device_id(&claims)?;
    let pair_code = req.pair_code.clone();
    let device_id_str = device_id.to_string();

    if state.auth_throttle.is_known_missing_pair_device(device_id) {
        return Err(ServerError::Unauthorized);
    }

    let now = chrono::Utc::now();
    let now_str = now.to_rfc3339();
    let mut tx = state.db.begin().await?;

    let device_exists = sqlx::query_as::<_, (i64,)>(
        r#"
        SELECT 1
        FROM client_devices
        WHERE device_id = $1
        "#,
    )
    .bind(&device_id_str)
    .fetch_optional(&mut *tx)
    .await?;

    if device_exists.is_none() {
        state.auth_throttle.remember_missing_pair_device(device_id);
        return Err(ServerError::Unauthorized);
    }

    state.auth_throttle.clear_missing_pair_device(device_id);
    validate_pair_code(&pair_code)?;

    if !state.auth_throttle.allow_pair_device(device_id) {
        return Err(ServerError::TooManyRequests(
            "Too many pairing attempts for this device".to_string(),
        ));
    }

    if !state
        .auth_throttle
        .allow_pair_code(device_id, pair_code.clone())
    {
        return Err(ServerError::TooManyRequests(
            "Too many pairing attempts for this device and code".to_string(),
        ));
    }

    let pairing: (String, String, String, i64, String) = sqlx::query_as(
        r#"
        SELECT pair_code, host_id, host_name, used, expires_at
        FROM pairing_codes
        WHERE pair_code = $1
        "#,
    )
    .bind(&pair_code)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ServerError::PairCodeExpired)?;

    if pairing.3 != 0 {
        return Err(ServerError::PairCodeUsed);
    }

    let expires_at = utils::parse_sqlite_timestamp(&pairing.4)
        .map_err(|e| ServerError::Internal(format!("Invalid expiration time: {}", e)))?;

    if now > expires_at.with_timezone(&chrono::Utc) {
        return Err(ServerError::PairCodeExpired);
    }

    let host_id = Uuid::parse_str(&pairing.1)
        .map_err(|e| ServerError::Internal(format!("Invalid host ID: {}", e)))?;
    let host_id_str = pairing.1.clone();
    let host_name = pairing.2.clone();

    let claim_result = sqlx::query(
        r#"
        UPDATE pairing_codes
        SET used = 1
        WHERE pair_code = $1 AND used = 0 AND expires_at > $2
        "#,
    )
    .bind(&pair_code)
    .bind(&now_str)
    .execute(&mut *tx)
    .await?;

    if claim_result.rows_affected() == 0 {
        let latest: (i64, String) = sqlx::query_as(
            r#"
            SELECT used, expires_at
            FROM pairing_codes
            WHERE pair_code = $1
            "#,
        )
        .bind(&pair_code)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(ServerError::PairCodeExpired)?;

        if latest.0 != 0 {
            return Err(ServerError::PairCodeUsed);
        }

        return Err(ServerError::PairCodeExpired);
    }

    sqlx::query(
        r#"
        UPDATE client_devices
        SET legacy_acl = 0
        WHERE device_id = $1
        "#,
    )
    .bind(&device_id_str)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE hosts
        SET pair_status = 'paired', pair_code = NULL, qr_payload = NULL, updated_at = $2
        WHERE host_id = $1
        "#,
    )
    .bind(&host_id_str)
    .bind(&now_str)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO device_host_access (device_id, host_id)
        SELECT $1, $2
        WHERE NOT EXISTS (
            SELECT 1 FROM device_host_access WHERE device_id = $1 AND host_id = $2
        )
        "#,
    )
    .bind(device_id.to_string())
    .bind(&host_id_str)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO device_session_access (device_id, session_id)
        SELECT $1, sessions.session_id
        FROM sessions
        WHERE sessions.host_id = $2
          AND NOT EXISTS (
              SELECT 1
              FROM device_session_access
              WHERE device_id = $1 AND session_id = sessions.session_id
          )
        "#,
    )
    .bind(device_id.to_string())
    .bind(&host_id_str)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let jwt_manager = JwtManager::new(&state.config.jwt_secret, state.config.jwt_expiration());
    let client_token = jwt_manager.create_client_token(device_id, &claims.name)?;
    let daemon_token = jwt_manager.create_daemon_token(host_id, &host_name)?;

    let _ = state
        .hub
        .send_to_daemon(
            &host_id,
            ve_shared::proto::DaemonMessage::Paired {
                host_id,
                daemon_token,
            },
        )
        .await;

    tracing::info!(%host_id, "Pairing completed");

    Ok(Json(PairResponse {
        host_id,
        host_name,
        token: client_token,
    }))
}

/// Generate a random high-entropy pairing secret
fn generate_pairing_secret() -> String {
    let secret: [u8; 32] = rand::random();
    hex::encode(secret)
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
