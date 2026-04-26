//! Session API integration tests

use super::commands::*;
use super::control::{handle_archived_rerun, validate_live_control_action, ControlRequest};
use super::messages::{MessageListQuery, SendMessageRequest};
use super::*;
use crate::config::Config;
use crate::db::{install_drivers, run_migrations, DbPool};
use crate::hub::Hub;
use crate::state::AppState;
use crate::validation::{ValidationError, MAX_IDEMPOTENCY_KEY_LENGTH};
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::Extension;
use std::sync::Arc;
use ve_shared::jwt::Claims;
use ve_shared::models::{ArchiveMetadata, ArchiveStatistics};
use ve_shared::types::CloseReason;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, DatabaseBackend};
    use crate::db::{install_drivers, run_migrations, DbPool};
    use crate::hub::Hub;
    use crate::state::AppState;
    use crate::validation::{ValidationError, MAX_IDEMPOTENCY_KEY_LENGTH};
    use axum::response::IntoResponse;
    use ve_shared::models::{ArchiveMetadata, ArchiveStatistics};

    fn test_config(database_url: String) -> Config {
        const TEST_JWT_SECRET: &str = "test_secret_for_unit_tests_only_32chars!";
        Config {
            listen_addr: "127.0.0.1:3000".parse().unwrap(),
            database_url,
            jwt_secret: TEST_JWT_SECRET.to_string(),
            jwt_expiration_secs: 3600,
            pair_code_ttl_secs: 300,
            heartbeat_interval_secs: 30,
            connection_timeout_secs: 60,
            data_dir: std::path::PathBuf::from("/tmp"),
            cors_origins: Vec::new(),
            ack_timeout_ms: 10000,
            ack_max_retries: 2,
            ack_retry_delay_ms: 500,
            permission_ttl_secs: 1800,
            permission_expiry_check_secs: 60,
            idempotency_ttl_secs: 86400,
            idempotency_cleanup_secs: 3600,
            log_format: "pretty".to_string(),
            log_level: "info".to_string(),
        }
    }

    async fn setup_state() -> Arc<AppState> {
        install_drivers();
        let temp_db =
            std::env::temp_dir().join(format!("ve-sessions-api-test-{}.db", Uuid::new_v4()));
        let database_url = format!("sqlite:{}?mode=rwc", temp_db.display());
        let pool = DbPool::connect(&database_url).await.unwrap();
        run_migrations(&pool, DatabaseBackend::Sqlite)
            .await
            .unwrap();

        let config = test_config(database_url);
        let jwt_manager = Arc::new(ve_shared::jwt::JwtManager::new(
            &config.jwt_secret,
            config.jwt_expiration(),
        ));
        Arc::new(AppState::new(pool, Hub::new(), config, jwt_manager))
    }

    async fn insert_host_workspace_and_archive(
        state: &Arc<AppState>,
        archived_session_id: Uuid,
        host_id: Uuid,
        workspace_id: Uuid,
        metadata_json: Option<String>,
    ) {
        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)")
            .bind(workspace_id.to_string())
            .bind(host_id.to_string())
            .bind("/tmp")
            .bind("tmp")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO session_archives (archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id, metadata_json, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(archived_session_id.to_string())
        .bind("archived")
        .bind(chrono::Utc::now().to_rfc3339())
        .bind("user_closed")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind(metadata_json)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();
    }

    async fn insert_archived_rerun_fixture(
        state: &Arc<AppState>,
        archived_session_id: Uuid,
        host_id: Uuid,
        workspace_id: Uuid,
        device_ids: &[Uuid],
    ) {
        let metadata = ArchiveMetadata {
            workspace_path: "/tmp".to_string(),
            workspace_display_name: Some("tmp".to_string()),
            agent_type: "claude_code".to_string(),
            closed_by: "server".to_string(),
            final_summary: None,
            claude_session_id: Some("claude-session-1".to_string()),
            statistics: Some(ArchiveStatistics {
                message_count: 0,
                event_count: 0,
                permission_count: 0,
                duration_seconds: 0,
            }),
            last_commit_sha: None,
            last_commit_message: None,
        };

        insert_host_workspace_and_archive(
            state,
            archived_session_id,
            host_id,
            workspace_id,
            Some(serde_json::to_string(&metadata).unwrap()),
        )
        .await;

        for device_id in device_ids {
            sqlx::query(
                "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
            )
            .bind(device_id.to_string())
            .bind("device")
            .bind("desktop")
            .bind("http://localhost")
            .execute(&state.db)
            .await
            .unwrap();

            sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
                .bind(device_id.to_string())
                .bind(host_id.to_string())
                .execute(&state.db)
                .await
                .unwrap();
        }

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
        )
        .bind(archived_session_id.to_string())
        .bind("archived")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        for device_id in device_ids {
            sqlx::query(
                "INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)",
            )
            .bind(device_id.to_string())
            .bind(archived_session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();
        }
    }

    #[tokio::test]
    async fn archive_session_with_metadata_returns_existing_archive_for_duplicate_calls() {
        let state = setup_state().await;
        let session_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
        )
        .bind(workspace_id.to_string())
        .bind(host_id.to_string())
        .bind("/tmp")
        .bind("tmp")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'running', $6, $6)",
        )
        .bind(session_id.to_string())
        .bind("test")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        let first_archive_id =
            archive_session_with_metadata(&state, session_id, CloseReason::UserClosed, None)
                .await
                .unwrap();
        let second_archive_id = archive_session_with_metadata(
            &state,
            session_id,
            CloseReason::Terminated,
            Some("ignored".to_string()),
        )
        .await
        .unwrap();

        assert_eq!(second_archive_id, first_archive_id);

        let archive_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM session_archives WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(archive_count.0, 1);

        let archive: (String, String) = sqlx::query_as(
            "SELECT close_reason, metadata_json FROM session_archives WHERE session_id = $1",
        )
        .bind(session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(archive.0, "user_closed");

        let metadata: ArchiveMetadata = serde_json::from_str(&archive.1).unwrap();
        assert_eq!(metadata.final_summary, None);

        let latest_summary: (Option<String>,) =
            sqlx::query_as("SELECT latest_summary FROM sessions WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(latest_summary.0, None);
    }

    #[test]
    fn live_control_rejects_rerun() {
        let error =
            validate_live_control_action("running", SessionControlAction::Rerun).unwrap_err();
        assert!(matches!(
            error,
            ServerError::Conflict(message)
                if message == "rerun is only supported for archived sessions; use restart for live sessions"
        ));
    }

    #[test]
    fn live_control_allows_restart() {
        assert!(validate_live_control_action("running", SessionControlAction::Restart).is_ok());
    }

    #[tokio::test]
    async fn archived_rerun_returns_internal_error_when_archive_metadata_is_invalid() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();

        insert_host_workspace_and_archive(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            Some("{not-json".to_string()),
        )
        .await;

        let session = (
            "archived".to_string(),
            host_id.to_string(),
            workspace_id.to_string(),
            "archived".to_string(),
            "claude_code".to_string(),
            None,
        );

        let before: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();

        let error = handle_archived_rerun(
            &state,
            "tr-test",
            Uuid::new_v4(),
            archived_session_id,
            "rerun",
            session,
        )
        .await
        .unwrap_err();

        let response = error.into_response();
        assert_eq!(
            response.status(),
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );

        let after: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(before.0, after.0);
    }

    #[tokio::test]
    async fn archived_rerun_returns_conflict_when_archive_metadata_has_no_claude_session_id() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let metadata = ArchiveMetadata {
            workspace_path: "/tmp".to_string(),
            workspace_display_name: Some("tmp".to_string()),
            agent_type: "claude_code".to_string(),
            closed_by: "server".to_string(),
            final_summary: None,
            claude_session_id: None,
            statistics: Some(ArchiveStatistics {
                message_count: 0,
                event_count: 0,
                permission_count: 0,
                duration_seconds: 0,
            }),
            last_commit_sha: None,
            last_commit_message: None,
        };

        insert_host_workspace_and_archive(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            Some(serde_json::to_string(&metadata).unwrap()),
        )
        .await;

        let session = (
            "archived".to_string(),
            host_id.to_string(),
            workspace_id.to_string(),
            "archived".to_string(),
            "claude_code".to_string(),
            None,
        );

        let before: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();

        let error = handle_archived_rerun(
            &state,
            "tr-test",
            Uuid::new_v4(),
            archived_session_id,
            "rerun",
            session,
        )
        .await
        .unwrap_err();

        let response = error.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::CONFLICT);

        let after: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(before.0, after.0);
    }

    #[allow(dead_code)]
    async fn insert_device_and_accessible_sessions(
        state: &Arc<AppState>,
        device_id: Uuid,
        host_id: Uuid,
        visible_session_id: Uuid,
        hidden_session_id: Uuid,
    ) {
        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        let workspace_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
        )
        .bind(workspace_id.to_string())
        .bind(host_id.to_string())
        .bind("/tmp")
        .bind("tmp")
        .execute(&state.db)
        .await
        .unwrap();

        for (session_id, title) in [
            (visible_session_id, "visible"),
            (hidden_session_id, "hidden"),
        ] {
            sqlx::query(
                r#"
                INSERT INTO sessions (
                    session_id, title, host_id, workspace_id, agent_type, status,
                    unread_event_count, pending_permission_count, can_resume_cross_device,
                    created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, 'running', 0, 0, 1, $6, $6)
                "#,
            )
            .bind(session_id.to_string())
            .bind(title)
            .bind(host_id.to_string())
            .bind(workspace_id.to_string())
            .bind("claude_code")
            .bind(chrono::Utc::now().to_rfc3339())
            .execute(&state.db)
            .await
            .unwrap();
        }

        sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(host_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(visible_session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn create_session_rejects_workspace_from_other_host() {
        let state = setup_state().await;
        let device_id = Uuid::new_v4();
        let allowed_host_id = Uuid::new_v4();
        let other_host_id = Uuid::new_v4();
        let foreign_workspace_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        for host_id in [allowed_host_id, other_host_id] {
            sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
                .bind(host_id.to_string())
                .bind("host")
                .bind("linux")
                .execute(&state.db)
                .await
                .unwrap();
        }

        sqlx::query(
            "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
        )
        .bind(foreign_workspace_id.to_string())
        .bind(other_host_id.to_string())
        .bind("/tmp/foreign")
        .bind("foreign")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(allowed_host_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        let error = create_session(
            ClientAccess { device_id },
            State(state.clone()),
            HeaderMap::new(),
            Json(CreateSessionRequest {
                idempotency_key: Uuid::new_v4().to_string(),
                host_id: allowed_host_id,
                workspace_id: foreign_workspace_id,
                title: "test".to_string(),
                initial_message: "hello".to_string(),
            }),
        )
        .await
        .unwrap_err();

        assert!(
            matches!(error, ServerError::NotFound(message) if message == format!("Workspace {}", foreign_workspace_id))
        );

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(count.0, 0);
    }

    #[tokio::test]
    async fn create_session_rejects_empty_idempotency_key() {
        let state = setup_state().await;

        let error = create_session(
            ClientAccess {
                device_id: Uuid::new_v4(),
            },
            State(state.clone()),
            HeaderMap::new(),
            Json(CreateSessionRequest {
                idempotency_key: "   ".to_string(),
                host_id: Uuid::new_v4(),
                workspace_id: Uuid::new_v4(),
                title: "test".to_string(),
                initial_message: "hello".to_string(),
            }),
        )
        .await
        .unwrap_err();

        assert!(matches!(
            error,
            ServerError::Validation(ValidationError::Empty {
                field: "idempotency_key",
            })
        ));

        let session_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(session_count.0, 0);
    }

    #[tokio::test]
    async fn create_session_rejects_too_long_idempotency_key() {
        let state = setup_state().await;

        let error = create_session(
            ClientAccess {
                device_id: Uuid::new_v4(),
            },
            State(state.clone()),
            HeaderMap::new(),
            Json(CreateSessionRequest {
                idempotency_key: "a".repeat(MAX_IDEMPOTENCY_KEY_LENGTH + 1),
                host_id: Uuid::new_v4(),
                workspace_id: Uuid::new_v4(),
                title: "test".to_string(),
                initial_message: "hello".to_string(),
            }),
        )
        .await
        .unwrap_err();

        assert!(matches!(
            error,
            ServerError::Validation(ValidationError::TooLong {
                field: "idempotency_key",
                max: MAX_IDEMPOTENCY_KEY_LENGTH,
            })
        ));
    }

    #[cfg(debug_assertions)]
    #[tokio::test]
    async fn create_session_is_strictly_idempotent_under_concurrency() {
        let state = setup_state().await;
        let device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let idempotency_key = Uuid::new_v4().to_string();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(4);

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
        )
        .bind(workspace_id.to_string())
        .bind(host_id.to_string())
        .bind("/tmp")
        .bind("tmp")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(host_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        state.hub.register_daemon(host_id, daemon_tx).await;
        let _race_hook = test_support::install_create_session_race_hook(idempotency_key.clone());

        let spawn_request = || {
            let state = state.clone();
            let idempotency_key = idempotency_key.clone();
            tokio::spawn(async move {
                create_session(
                    ClientAccess { device_id },
                    State(state),
                    HeaderMap::new(),
                    Json(CreateSessionRequest {
                        idempotency_key,
                        host_id,
                        workspace_id,
                        title: "test".to_string(),
                        initial_message: "hello".to_string(),
                    }),
                )
                .await
            })
        };

        let first_task = spawn_request();
        let second_task = spawn_request();

        let outbound = daemon_rx.recv().await.unwrap();
        assert_eq!(outbound.r#type, "create_session");
        let session_id = outbound.payload["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: outbound.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let first = first_task.await.unwrap().unwrap().0;
        let second = second_task.await.unwrap().unwrap().0;
        assert_eq!(first.session_id, second.session_id);
        assert_eq!(first.session_id, session_id);

        let session_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(session_count.0, 1);

        let message_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM session_messages WHERE session_id = $1")
                .bind(first.session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(message_count.0, 1);

        let idempotency_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM idempotency_keys WHERE key = $1")
                .bind(&idempotency_key)
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(idempotency_count.0, 1);

        let access_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM device_session_access WHERE device_id = $1 AND session_id = $2",
        )
        .bind(device_id.to_string())
        .bind(first.session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(access_count.0, 1);

        assert!(daemon_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn create_session_waits_for_daemon_ack_before_returning_success() {
        let state = setup_state().await;
        let device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
        )
        .bind(workspace_id.to_string())
        .bind(host_id.to_string())
        .bind("/tmp")
        .bind("tmp")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(host_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        state.hub.register_daemon(host_id, daemon_tx).await;

        let state_for_request = state.clone();
        let request_task = tokio::spawn(async move {
            create_session(
                ClientAccess { device_id },
                State(state_for_request),
                HeaderMap::new(),
                Json(CreateSessionRequest {
                    idempotency_key: Uuid::new_v4().to_string(),
                    host_id,
                    workspace_id,
                    title: "test".to_string(),
                    initial_message: "hello".to_string(),
                }),
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "create_session");
        let session_id = queued.payload["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        let status_before: (String,) =
            sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(status_before.0, "running");
        assert!(!request_task.is_finished());

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: queued.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let response = request_task.await.unwrap().unwrap().0;
        assert_eq!(response.session_id, session_id);
    }

    #[tokio::test]
    async fn create_session_marks_session_error_when_daemon_rejects_create() {
        let state = setup_state().await;
        let device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
        )
        .bind(workspace_id.to_string())
        .bind(host_id.to_string())
        .bind("/tmp")
        .bind("tmp")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(host_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        state.hub.register_daemon(host_id, daemon_tx).await;

        let state_for_request = state.clone();
        let request_task = tokio::spawn(async move {
            create_session(
                ClientAccess { device_id },
                State(state_for_request),
                HeaderMap::new(),
                Json(CreateSessionRequest {
                    idempotency_key: Uuid::new_v4().to_string(),
                    host_id,
                    workspace_id,
                    title: "test".to_string(),
                    initial_message: "hello".to_string(),
                }),
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "create_session");
        let session_id = queued.payload["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        state
            .hub
            .complete_with_error(ve_shared::proto::ErrorPayload {
                request_id: queued.request_id.clone().unwrap(),
                error_code: "session_create_failed".to_string(),
                error_message: "daemon rejected create".to_string(),
            })
            .await;

        let error = request_task.await.unwrap().unwrap_err();
        assert!(
            matches!(error, ServerError::Conflict(message) if message == "Daemon command failed")
        );

        let row: (String, Option<String>) =
            sqlx::query_as("SELECT status, latest_summary FROM sessions WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(row.0, "error");
        assert_eq!(
            row.1.as_deref(),
            Some("Failed to queue create_session request to daemon")
        );
    }

    #[tokio::test]
    async fn archived_rerun_marks_new_session_error_when_daemon_rejects_rerun() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let rerun_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);

        insert_archived_rerun_fixture(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            &[rerun_device_id],
        )
        .await;
        state.hub.register_daemon(host_id, daemon_tx).await;

        let state_for_request = state.clone();
        let request_task = tokio::spawn(async move {
            control_session(
                State(state_for_request),
                Extension(Claims::for_client(
                    rerun_device_id,
                    "device",
                    chrono::Duration::hours(1),
                )),
                HeaderMap::new(),
                Path(archived_session_id),
                Json(ControlRequest {
                    action: "rerun".to_string(),
                }),
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "rerun_session");
        let new_session_id = queued.payload["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: queued.request_id.clone().unwrap(),
                success: false,
                error: Some("daemon rejected rerun".to_string()),
            })
            .await;

        let error = request_task.await.unwrap().unwrap_err();
        assert!(
            matches!(error, ServerError::Conflict(message) if message == "Daemon command failed")
        );

        let row: (String, Option<String>) =
            sqlx::query_as("SELECT status, latest_summary FROM sessions WHERE session_id = $1")
                .bind(new_session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(row.0, "error");
        assert_eq!(
            row.1.as_deref(),
            Some("Failed to queue rerun request to daemon")
        );
    }

    #[tokio::test]
    async fn archived_rerun_grants_new_session_to_requester() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let new_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);
        let metadata = ArchiveMetadata {
            workspace_path: "/tmp".to_string(),
            workspace_display_name: Some("tmp".to_string()),
            agent_type: "claude_code".to_string(),
            closed_by: "server".to_string(),
            final_summary: None,
            claude_session_id: Some("claude-session-1".to_string()),
            statistics: Some(ArchiveStatistics {
                message_count: 0,
                event_count: 0,
                permission_count: 0,
                duration_seconds: 0,
            }),
            last_commit_sha: None,
            last_commit_message: None,
        };

        insert_host_workspace_and_archive(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            Some(serde_json::to_string(&metadata).unwrap()),
        )
        .await;
        state.hub.register_daemon(host_id, daemon_tx).await;

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(new_device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
        )
        .bind(archived_session_id.to_string())
        .bind("archived")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(new_device_id.to_string())
            .bind(archived_session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        let session = (
            "archived".to_string(),
            host_id.to_string(),
            workspace_id.to_string(),
            "archived".to_string(),
            "claude_code".to_string(),
            None,
        );

        let state_for_request = state.clone();
        let request_task = tokio::spawn(async move {
            handle_archived_rerun(
                &state_for_request,
                "tr-test",
                new_device_id,
                archived_session_id,
                "rerun",
                session,
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "rerun_session");
        let new_session_id = queued.payload["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        let status: (String,) = sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
            .bind(new_session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(status.0, "dispatching");

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: queued.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let response = request_task.await.unwrap().unwrap().0;
        let response_session_id = response["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();
        assert_eq!(response_session_id, new_session_id);

        let access_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM device_session_access WHERE device_id = $1 AND session_id = $2",
        )
        .bind(new_device_id.to_string())
        .bind(new_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(access_count.0, 1);
    }

    #[tokio::test]
    async fn archived_rerun_does_not_grant_new_session_to_host_only_devices() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let inherited_device_id = Uuid::new_v4();
        let host_only_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);
        let metadata = ArchiveMetadata {
            workspace_path: "/tmp".to_string(),
            workspace_display_name: Some("tmp".to_string()),
            agent_type: "claude_code".to_string(),
            closed_by: "server".to_string(),
            final_summary: None,
            claude_session_id: Some("claude-session-1".to_string()),
            statistics: Some(ArchiveStatistics {
                message_count: 0,
                event_count: 0,
                permission_count: 0,
                duration_seconds: 0,
            }),
            last_commit_sha: None,
            last_commit_message: None,
        };

        insert_host_workspace_and_archive(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            Some(serde_json::to_string(&metadata).unwrap()),
        )
        .await;
        state.hub.register_daemon(host_id, daemon_tx).await;

        for device_id in [inherited_device_id, host_only_device_id] {
            sqlx::query(
                "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
            )
            .bind(device_id.to_string())
            .bind("device")
            .bind("desktop")
            .bind("http://localhost")
            .execute(&state.db)
            .await
            .unwrap();

            sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
                .bind(device_id.to_string())
                .bind(host_id.to_string())
                .execute(&state.db)
                .await
                .unwrap();
        }

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
        )
        .bind(archived_session_id.to_string())
        .bind("archived")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(inherited_device_id.to_string())
            .bind(archived_session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        let session = (
            "archived".to_string(),
            host_id.to_string(),
            workspace_id.to_string(),
            "archived".to_string(),
            "claude_code".to_string(),
            None,
        );

        let state_for_request = state.clone();
        let request_task = tokio::spawn(async move {
            handle_archived_rerun(
                &state_for_request,
                "tr-test",
                inherited_device_id,
                archived_session_id,
                "rerun",
                session,
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "rerun_session");
        let new_session_id = queued.payload["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: queued.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let response = request_task.await.unwrap().unwrap().0;
        let response_session_id = response["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();
        assert_eq!(response_session_id, new_session_id);

        let inherited_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM device_session_access WHERE device_id = $1 AND session_id = $2",
        )
        .bind(inherited_device_id.to_string())
        .bind(new_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(inherited_count.0, 1);

        let host_only_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM device_session_access WHERE device_id = $1 AND session_id = $2",
        )
        .bind(host_only_device_id.to_string())
        .bind(new_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(host_only_count.0, 0);
    }

    #[tokio::test]
    async fn archived_rerun_via_control_requires_current_host_access_even_with_archived_session_acl(
    ) {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let inherited_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (_daemon_tx, mut daemon_rx) =
            tokio::sync::mpsc::channel::<ve_shared::proto::WsEnvelope>(1);
        let metadata = ArchiveMetadata {
            workspace_path: "/tmp".to_string(),
            workspace_display_name: Some("tmp".to_string()),
            agent_type: "claude_code".to_string(),
            closed_by: "server".to_string(),
            final_summary: None,
            claude_session_id: Some("claude-session-1".to_string()),
            statistics: Some(ArchiveStatistics {
                message_count: 0,
                event_count: 0,
                permission_count: 0,
                duration_seconds: 0,
            }),
            last_commit_sha: None,
            last_commit_message: None,
        };

        insert_host_workspace_and_archive(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            Some(serde_json::to_string(&metadata).unwrap()),
        )
        .await;

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(inherited_device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
        )
        .bind(archived_session_id.to_string())
        .bind("archived")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(inherited_device_id.to_string())
            .bind(archived_session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        let before_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();

        let error = control_session(
            State(state.clone()),
            Extension(Claims::for_client(
                inherited_device_id,
                "device",
                chrono::Duration::hours(1),
            )),
            HeaderMap::new(),
            Path(archived_session_id),
            Json(ControlRequest {
                action: "rerun".to_string(),
            }),
        )
        .await
        .unwrap_err();

        assert!(
            matches!(error, ServerError::NotFound(message) if message == format!("Host {}", host_id))
        );

        let after_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(after_count.0, before_count.0);

        assert!(daemon_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn archived_rerun_via_control_grants_new_session_only_to_rerun_requester() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let archived_only_device_id = Uuid::new_v4();
        let rerun_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);
        let metadata = ArchiveMetadata {
            workspace_path: "/tmp".to_string(),
            workspace_display_name: Some("tmp".to_string()),
            agent_type: "claude_code".to_string(),
            closed_by: "server".to_string(),
            final_summary: None,
            claude_session_id: Some("claude-session-1".to_string()),
            statistics: Some(ArchiveStatistics {
                message_count: 0,
                event_count: 0,
                permission_count: 0,
                duration_seconds: 0,
            }),
            last_commit_sha: None,
            last_commit_message: None,
        };

        insert_host_workspace_and_archive(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            Some(serde_json::to_string(&metadata).unwrap()),
        )
        .await;
        state.hub.register_daemon(host_id, daemon_tx).await;

        for device_id in [archived_only_device_id, rerun_device_id] {
            sqlx::query(
                "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
            )
            .bind(device_id.to_string())
            .bind("device")
            .bind("desktop")
            .bind("http://localhost")
            .execute(&state.db)
            .await
            .unwrap();

            sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
                .bind(device_id.to_string())
                .bind(host_id.to_string())
                .execute(&state.db)
                .await
                .unwrap();
        }

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
        )
        .bind(archived_session_id.to_string())
        .bind("archived")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        for device_id in [archived_only_device_id, rerun_device_id] {
            sqlx::query(
                "INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)",
            )
            .bind(device_id.to_string())
            .bind(archived_session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();
        }

        let state_for_request = state.clone();
        let request_task = tokio::spawn(async move {
            control_session(
                State(state_for_request),
                Extension(Claims::for_client(
                    rerun_device_id,
                    "device",
                    chrono::Duration::hours(1),
                )),
                HeaderMap::new(),
                Path(archived_session_id),
                Json(ControlRequest {
                    action: "rerun".to_string(),
                }),
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "rerun_session");
        let new_session_id = queued.payload["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: queued.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let rerun_response = request_task.await.unwrap().unwrap().0;
        let response_session_id = rerun_response["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();
        assert_eq!(response_session_id, new_session_id);

        let rerun_device_sessions = list_sessions(
            SessionCollectionAccess {
                device_id: rerun_device_id,
                host_id: Some(host_id),
            },
            State(state.clone()),
        )
        .await
        .unwrap()
        .0;
        assert!(rerun_device_sessions
            .iter()
            .any(|session| session.session_id == new_session_id));

        let archived_only_sessions = list_sessions(
            SessionCollectionAccess {
                device_id: archived_only_device_id,
                host_id: Some(host_id),
            },
            State(state.clone()),
        )
        .await
        .unwrap()
        .0;
        assert!(!archived_only_sessions
            .iter()
            .any(|session| session.session_id == new_session_id));
    }

    #[tokio::test]
    async fn archived_rerun_persists_origin_link_on_first_live_rerun() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let rerun_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);

        insert_archived_rerun_fixture(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            &[rerun_device_id],
        )
        .await;
        state.hub.register_daemon(host_id, daemon_tx).await;

        let state_for_request = state.clone();
        let request_task = tokio::spawn(async move {
            control_session(
                State(state_for_request),
                Extension(Claims::for_client(
                    rerun_device_id,
                    "device",
                    chrono::Duration::hours(1),
                )),
                HeaderMap::new(),
                Path(archived_session_id),
                Json(ControlRequest {
                    action: "rerun".to_string(),
                }),
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "rerun_session");
        let new_session_id = queued.payload["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        let rerun_row_before_ack: (String, String, String) = sqlx::query_as(
            r#"
            SELECT status, claude_session_id, rerun_from_session_id
            FROM sessions
            WHERE session_id = $1
            "#,
        )
        .bind(new_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(rerun_row_before_ack.0, "dispatching");
        assert_eq!(rerun_row_before_ack.1, "claude-session-1");
        assert_eq!(rerun_row_before_ack.2, archived_session_id.to_string());

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: queued.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let rerun_response = request_task.await.unwrap().unwrap().0;
        let response_session_id = rerun_response["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();
        assert_eq!(response_session_id, new_session_id);

        let rerun_row_after_ack: (String, String, String) = sqlx::query_as(
            r#"
            SELECT status, claude_session_id, rerun_from_session_id
            FROM sessions
            WHERE session_id = $1
            "#,
        )
        .bind(new_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(rerun_row_after_ack.0, "pending");
        assert_eq!(rerun_row_after_ack.1, "claude-session-1");
        assert_eq!(rerun_row_after_ack.2, archived_session_id.to_string());

        let active_rerun_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sessions WHERE rerun_from_session_id = $1 AND status NOT IN ('dispatching', 'archived', 'error')",
        )
        .bind(archived_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(active_rerun_count.0, 1);

        let access_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM device_session_access WHERE device_id = $1 AND session_id = $2",
        )
        .bind(rerun_device_id.to_string())
        .bind(new_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(access_count.0, 1);
    }

    #[tokio::test]
    async fn archived_rerun_waits_for_daemon_ack_before_marking_pending() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let rerun_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);

        insert_archived_rerun_fixture(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            &[rerun_device_id],
        )
        .await;
        state.hub.register_daemon(host_id, daemon_tx).await;

        let state_for_request = state.clone();
        let request_task = tokio::spawn(async move {
            control_session(
                State(state_for_request),
                Extension(Claims::for_client(
                    rerun_device_id,
                    "device",
                    chrono::Duration::hours(1),
                )),
                HeaderMap::new(),
                Path(archived_session_id),
                Json(ControlRequest {
                    action: "rerun".to_string(),
                }),
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "rerun_session");
        let new_session_id = queued.payload["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        let status_before: (String,) =
            sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
                .bind(new_session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(status_before.0, "dispatching");
        assert!(!request_task.is_finished());

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: queued.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let response = request_task.await.unwrap().unwrap().0;
        let response_session_id = response["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();
        assert_eq!(response_session_id, new_session_id);

        let status_after: (String,) =
            sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
                .bind(new_session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(status_after.0, "pending");
    }

    #[tokio::test]
    async fn archived_rerun_returns_existing_active_rerun_on_repeat_request() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let first_device_id = Uuid::new_v4();
        let second_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(2);

        insert_archived_rerun_fixture(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            &[first_device_id, second_device_id],
        )
        .await;
        state.hub.register_daemon(host_id, daemon_tx).await;

        let state_for_first = state.clone();
        let first_task = tokio::spawn(async move {
            control_session(
                State(state_for_first),
                Extension(Claims::for_client(
                    first_device_id,
                    "device",
                    chrono::Duration::hours(1),
                )),
                HeaderMap::new(),
                Path(archived_session_id),
                Json(ControlRequest {
                    action: "rerun".to_string(),
                }),
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "rerun_session");
        let first_session_id = queued.payload["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: queued.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let first_response = first_task.await.unwrap().unwrap().0;
        let first_response_session_id = first_response["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();
        assert_eq!(first_response_session_id, first_session_id);

        let second_response = control_session(
            State(state.clone()),
            Extension(Claims::for_client(
                second_device_id,
                "device",
                chrono::Duration::hours(1),
            )),
            HeaderMap::new(),
            Path(archived_session_id),
            Json(ControlRequest {
                action: "rerun".to_string(),
            }),
        )
        .await
        .unwrap()
        .0;
        let second_session_id = second_response["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        assert_eq!(second_session_id, first_session_id);

        let active_rerun_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sessions WHERE rerun_from_session_id = $1 AND status NOT IN ('dispatching', 'archived', 'error')",
        )
        .bind(archived_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(active_rerun_count.0, 1);

        let second_device_access: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM device_session_access WHERE device_id = $1 AND session_id = $2",
        )
        .bind(second_device_id.to_string())
        .bind(first_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(second_device_access.0, 1);
    }

    #[tokio::test]
    async fn archived_rerun_returns_conflict_while_existing_rerun_is_dispatching() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let first_device_id = Uuid::new_v4();
        let second_device_id = Uuid::new_v4();
        let dispatching_session_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let now = chrono::Utc::now().to_rfc3339();

        insert_archived_rerun_fixture(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            &[first_device_id, second_device_id],
        )
        .await;

        sqlx::query(
            r#"
            INSERT INTO sessions (
                session_id, title, host_id, workspace_id, agent_type, status,
                claude_session_id, rerun_from_session_id, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, 'dispatching', $6, $7, $8, $8)
            "#,
        )
        .bind(dispatching_session_id.to_string())
        .bind("dispatching rerun")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind("claude-session-1")
        .bind(archived_session_id.to_string())
        .bind(&now)
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(first_device_id.to_string())
            .bind(dispatching_session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        let error = control_session(
            State(state.clone()),
            Extension(Claims::for_client(
                second_device_id,
                "device",
                chrono::Duration::hours(1),
            )),
            HeaderMap::new(),
            Path(archived_session_id),
            Json(ControlRequest {
                action: "rerun".to_string(),
            }),
        )
        .await
        .unwrap_err();

        assert!(matches!(
            error,
            ServerError::Conflict(message)
                if message == "Archived session rerun is still dispatching; retry shortly"
        ));

        let second_device_access: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM device_session_access WHERE device_id = $1 AND session_id = $2",
        )
        .bind(second_device_id.to_string())
        .bind(dispatching_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(second_device_access.0, 0);
    }

    #[tokio::test]
    async fn archived_rerun_marks_new_session_error_when_daemon_queue_fails() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let inherited_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let metadata = ArchiveMetadata {
            workspace_path: "/tmp".to_string(),
            workspace_display_name: Some("tmp".to_string()),
            agent_type: "claude_code".to_string(),
            closed_by: "server".to_string(),
            final_summary: None,
            claude_session_id: Some("claude-session-1".to_string()),
            statistics: Some(ArchiveStatistics {
                message_count: 0,
                event_count: 0,
                permission_count: 0,
                duration_seconds: 0,
            }),
            last_commit_sha: None,
            last_commit_message: None,
        };

        insert_host_workspace_and_archive(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            Some(serde_json::to_string(&metadata).unwrap()),
        )
        .await;

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(inherited_device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
            .bind(inherited_device_id.to_string())
            .bind(host_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
        )
        .bind(archived_session_id.to_string())
        .bind("archived")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(inherited_device_id.to_string())
            .bind(archived_session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        let before_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();

        let error = control_session(
            State(state.clone()),
            Extension(Claims::for_client(
                inherited_device_id,
                "device",
                chrono::Duration::hours(1),
            )),
            HeaderMap::new(),
            Path(archived_session_id),
            Json(ControlRequest {
                action: "rerun".to_string(),
            }),
        )
        .await
        .unwrap_err();

        assert!(matches!(
            error,
            ServerError::Conflict(message)
                if message == "Failed to queue rerun request to daemon"
        ));

        let after_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(after_count.0, before_count.0 + 1);

        let rerun_row: (String, Option<String>) = sqlx::query_as(
            r#"
            SELECT status, latest_summary
            FROM sessions
            WHERE host_id = $1 AND workspace_id = $2 AND session_id != $3
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind(archived_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(rerun_row.0, "error");
        assert_eq!(
            rerun_row.1.as_deref(),
            Some("Failed to queue rerun request to daemon")
        );
    }

    #[tokio::test]
    async fn close_session_waits_for_daemon_ack_before_creating_archive() {
        let state = setup_state().await;
        let device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
        )
        .bind(workspace_id.to_string())
        .bind(host_id.to_string())
        .bind("/tmp")
        .bind("tmp")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'running', $6, $6)",
        )
        .bind(session_id.to_string())
        .bind("test")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        state.hub.register_daemon(host_id, daemon_tx).await;

        let state_for_request = state.clone();
        let claims = Claims::for_client(device_id, "device", chrono::Duration::hours(1));
        let request_task = tokio::spawn(async move {
            close_session(
                State(state_for_request),
                Extension(claims),
                HeaderMap::new(),
                Path(session_id),
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "close_session");
        assert_eq!(
            queued.request_id.as_deref().map(|s| s.is_empty()),
            Some(false)
        );
        assert_eq!(
            queued.payload["session_id"],
            serde_json::json!(session_id.to_string())
        );

        let status_before: (String,) =
            sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(status_before.0, "running");

        let archive_count_before: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM session_archives WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(archive_count_before.0, 0);
        assert!(!request_task.is_finished());

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: queued.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let response = request_task.await.unwrap().unwrap().0;

        assert_eq!(response["success"], serde_json::json!(true));
        assert_eq!(response["close_requested"], serde_json::json!(true));
        assert!(response.get("archive_id").is_none());

        let status: (String,) = sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(status.0, "running");

        let archive_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM session_archives WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(archive_count.0, 0);
    }

    #[tokio::test]
    async fn close_session_returns_conflict_when_archive_row_is_missing_for_archived_session() {
        let state = setup_state().await;
        let device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
        )
        .bind(workspace_id.to_string())
        .bind(host_id.to_string())
        .bind("/tmp")
        .bind("tmp")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
        )
        .bind(session_id.to_string())
        .bind("test")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        let claims = Claims::for_client(device_id, "device", chrono::Duration::hours(1));
        let error = close_session(
            State(state.clone()),
            Extension(claims),
            HeaderMap::new(),
            Path(session_id),
        )
        .await
        .unwrap_err();

        assert!(matches!(
            error,
            ServerError::Conflict(message)
                if message == format!("Session {} is archived without an archive record", session_id)
        ));
    }

    #[tokio::test]
    async fn list_messages_rejects_page_zero() {
        let state = setup_state().await;
        let device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
        )
        .bind(workspace_id.to_string())
        .bind(host_id.to_string())
        .bind("/tmp")
        .bind("tmp")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'running', $6, $6)",
        )
        .bind(session_id.to_string())
        .bind("test")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(&now)
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        let error = list_messages(
            State(state.clone()),
            Extension(Claims::for_client(
                device_id,
                "device",
                chrono::Duration::hours(1),
            )),
            Path(session_id),
            Query(MessageListQuery {
                page: Some(0),
                limit: None,
            }),
        )
        .await
        .unwrap_err();

        assert!(
            matches!(error, ServerError::BadRequest(message) if message == "page must be greater than 0")
        );
    }
}
