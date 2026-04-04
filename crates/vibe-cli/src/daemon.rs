use std::{
    fs,
    process::{Command, Stdio},
    sync::Arc,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use axum::{Json, Router, extract::State, routing::post};
use tokio::{
    sync::{Mutex, oneshot},
    time::sleep,
};

use crate::{
    api::CliApiClient,
    config::Config,
    credentials::require_credentials,
    persistence::{
        DaemonInstallState, DaemonRegistry, DaemonSessionRecord, DaemonState, LocalSessionState,
        clear_daemon_install, clear_daemon_registry, clear_daemon_state, now_ms,
        read_daemon_install, read_daemon_registry, read_daemon_state, read_settings,
        write_daemon_install, write_daemon_registry, write_daemon_state, write_settings,
    },
    utils::machine_metadata::{build_machine_metadata, detect_resume_support},
};

#[derive(Clone)]
struct DaemonControlState {
    config: Config,
    sessions: Arc<Mutex<DaemonRegistry>>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

#[derive(serde::Deserialize)]
struct StopSessionBody {
    session_id: String,
}

pub async fn start_daemon(config: &Config) -> Result<DaemonState> {
    if let Some(state) = daemon_status(config)? {
        return Ok(state);
    }

    let current_exe = std::env::current_exe().context("failed to resolve current executable")?;
    let _child = Command::new(current_exe)
        .arg("__daemon_serve")
        .env("VIBE_SERVER_URL", &config.server_url)
        .env("VIBE_WEBAPP_URL", &config.webapp_url)
        .env("VIBE_HOME_DIR", &config.home_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to start daemon process")?;

    for _ in 0..50 {
        if let Some(state) = daemon_status(config)? {
            return Ok(state);
        }
        sleep(Duration::from_millis(100)).await;
    }

    bail!("daemon did not become ready in time")
}

pub fn daemon_status(config: &Config) -> Result<Option<DaemonState>> {
    let Some(state) = read_daemon_state(config)? else {
        return Ok(None);
    };
    if is_pid_alive(state.pid) {
        return Ok(Some(state));
    }
    clear_daemon_state(config)?;
    clear_daemon_registry(config)?;
    Ok(None)
}

pub async fn daemon_list_sessions(config: &Config) -> Result<Vec<DaemonSessionRecord>> {
    let registry: serde_json::Value = daemon_post(config, "/list", serde_json::json!({})).await?;
    Ok(registry
        .get("sessions")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|value| serde_json::from_value(value.clone()).ok())
        .collect())
}

pub async fn daemon_notify_session_started(
    config: &Config,
    session: &LocalSessionState,
    host_pid: Option<u32>,
    provider_pid: Option<u32>,
) -> Result<()> {
    let _response: serde_json::Value = daemon_post(
        config,
        "/session-started",
        serde_json::json!(DaemonSessionRecord {
            session_id: session.server_session_id.clone(),
            provider: session.provider.clone(),
            provider_session_id: session.provider_session_id.clone(),
            tag: session.tag.clone(),
            working_dir: session.working_dir.clone(),
            started_at: session.created_at,
            started_by: "terminal".into(),
            host_pid,
            provider_pid,
        }),
    )
    .await?;
    Ok(())
}

pub async fn daemon_stop_session(config: &Config, session_id: &str) -> Result<bool> {
    let response: serde_json::Value = daemon_post(
        config,
        "/stop-session",
        serde_json::json!({ "session_id": session_id }),
    )
    .await?;
    Ok(response.get("success").and_then(serde_json::Value::as_bool) == Some(true))
}

pub async fn stop_daemon(config: &Config) -> Result<bool> {
    let Some(state) = daemon_status(config)? else {
        return Ok(false);
    };

    if state.http_port.is_some() {
        if daemon_post::<serde_json::Value>(config, "/stop", serde_json::json!({}))
            .await
            .is_ok()
        {
            for _ in 0..20 {
                if daemon_status(config)?.is_none() {
                    return Ok(true);
                }
                sleep(Duration::from_millis(100)).await;
            }
        }
    }

    let status = Command::new("kill")
        .arg("-TERM")
        .arg(state.pid.to_string())
        .status()
        .context("failed to invoke kill for daemon shutdown")?;
    if !status.success() {
        bail!("failed to stop daemon process {}", state.pid);
    }
    clear_daemon_state(config)?;
    clear_daemon_registry(config)?;
    Ok(true)
}

pub fn install_daemon(config: &Config) -> Result<DaemonInstallState> {
    let current_exe = std::env::current_exe().context("failed to resolve current executable")?;
    if let Some(parent) = config.daemon_launcher_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let script = format!(
        "#!/usr/bin/env bash\nexec \"{}\" __daemon_serve \"$@\"\n",
        current_exe.display()
    );
    fs::write(&config.daemon_launcher_path, script)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(&config.daemon_launcher_path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&config.daemon_launcher_path, permissions)?;
    }

    let install = DaemonInstallState {
        launcher_path: config.daemon_launcher_path.display().to_string(),
        executable_path: current_exe.display().to_string(),
        installed_at: now_ms(),
    };
    write_daemon_install(config, &install)?;
    Ok(install)
}

pub async fn uninstall_daemon(config: &Config) -> Result<bool> {
    let _ = stop_daemon(config).await;
    match fs::remove_file(&config.daemon_launcher_path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    clear_daemon_install(config)?;
    Ok(true)
}

pub fn daemon_install_status(config: &Config) -> Result<Option<DaemonInstallState>> {
    Ok(read_daemon_install(config)?)
}

pub async fn run_daemon_service(config: Config) -> Result<()> {
    let credentials = require_credentials(&config)?;
    let machine_id = ensure_machine_id(&config)?;
    let daemon_started_at = now_ms();
    let std_listener = std::net::TcpListener::bind("127.0.0.1:0")
        .context("failed to bind daemon control listener")?;
    let http_port = std_listener.local_addr()?.port();
    std_listener.set_nonblocking(true)?;
    let listener = tokio::net::TcpListener::from_std(std_listener)?;
    let api = CliApiClient::new(config.clone(), credentials);
    let initial_machine_metadata = machine_metadata(&config);
    let machine = api
        .create_or_load_machine(
            &machine_id,
            &initial_machine_metadata,
            Some(&daemon_state_json(
                "running",
                daemon_started_at,
                Some(http_port),
            )),
        )
        .await?;
    let machine_sync = api.create_machine_sync_client(machine).await?;
    machine_sync
        .sync_machine_metadata_with_retry(&initial_machine_metadata, 3, Duration::from_millis(200))
        .await
        .context("failed to synchronize daemon machine metadata")?;

    let sessions = Arc::new(Mutex::new(read_daemon_registry(&config)?));
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel();

    write_daemon_state(
        &config,
        &DaemonState {
            pid: std::process::id(),
            http_port: Some(http_port),
            started_at: daemon_started_at,
            heartbeat_at: now_ms(),
        },
    )?;
    write_daemon_registry(&config, &sessions.lock().await.clone())?;

    let control_state = DaemonControlState {
        config: config.clone(),
        sessions: sessions.clone(),
        shutdown_tx: Arc::new(Mutex::new(Some(shutdown_tx))),
    };

    let server = tokio::spawn(async move {
        let app = Router::new()
            .route("/status", post(control_status))
            .route("/list", post(control_list))
            .route("/session-started", post(control_session_started))
            .route("/stop-session", post(control_stop_session))
            .route("/stop", post(control_stop))
            .with_state(control_state);
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = (&mut shutdown_rx).await;
            })
            .await
            .ok();
    });

    loop {
        tokio::select! {
            _ = sleep(Duration::from_secs(1)) => {
                let current_machine_metadata = machine_metadata(&config);
                machine_sync
                    .sync_machine_metadata_with_retry(
                        &current_machine_metadata,
                        3,
                        Duration::from_millis(200),
                    )
                    .await
                    .context("failed to refresh daemon machine metadata")?;
                machine_sync.machine_alive().await?;
                machine_sync
                    .update_daemon_state(&daemon_state_json("running", daemon_started_at, Some(http_port)))
                    .await?;
                write_daemon_state(
                    &config,
                    &DaemonState {
                        pid: std::process::id(),
                        http_port: Some(http_port),
                        started_at: daemon_started_at,
                        heartbeat_at: now_ms(),
                    },
                )?;
                write_daemon_registry(&config, &sessions.lock().await.clone())?;
            }
            _ = server_handle_finished(&server) => {
                let _ = machine_sync
                    .update_daemon_state(&daemon_state_json(
                        "shutting-down",
                        daemon_started_at,
                        Some(http_port),
                    ))
                    .await;
                clear_daemon_state(&config)?;
                clear_daemon_registry(&config)?;
                return Ok(());
            }
        }
    }
}

async fn control_status(State(state): State<DaemonControlState>) -> Json<serde_json::Value> {
    let daemon_state = read_daemon_state(&state.config).ok().flatten();
    Json(serde_json::json!({
        "running": daemon_state.is_some(),
        "state": daemon_state,
    }))
}

async fn control_list(State(state): State<DaemonControlState>) -> Json<serde_json::Value> {
    let sessions = state.sessions.lock().await;
    Json(serde_json::json!({
        "sessions": sessions.sessions.values().cloned().collect::<Vec<_>>(),
    }))
}

async fn control_session_started(
    State(state): State<DaemonControlState>,
    Json(body): Json<DaemonSessionRecord>,
) -> Json<serde_json::Value> {
    let mut sessions = state.sessions.lock().await;
    sessions
        .sessions
        .insert(body.session_id.clone(), body.clone());
    let _ = write_daemon_registry(&state.config, &sessions);
    Json(serde_json::json!({ "success": true }))
}

async fn control_stop_session(
    State(state): State<DaemonControlState>,
    Json(body): Json<StopSessionBody>,
) -> Json<serde_json::Value> {
    let tracked = {
        let sessions = state.sessions.lock().await;
        sessions.sessions.get(&body.session_id).cloned()
    };
    let credentials = match require_credentials(&state.config) {
        Ok(credentials) => credentials,
        Err(error) => {
            return Json(serde_json::json!({ "success": false, "error": error.to_string() }));
        }
    };
    let api = CliApiClient::new(state.config.clone(), credentials);
    let remote_stopped = match api.list_sessions().await {
        Ok(sessions) => {
            if let Some(session) = sessions
                .into_iter()
                .find(|session| session.id == body.session_id)
            {
                match api.create_session_client(&session).await {
                    Ok(client) => {
                        let _ = client.wait_for_connect(Duration::from_secs(10)).await;
                        let stopped = client.send_stop().await.is_ok();
                        client.close().await;
                        stopped
                    }
                    Err(_) => false,
                }
            } else {
                false
            }
        }
        Err(_) => false,
    };
    let local_stopped = tracked
        .as_ref()
        .and_then(|record| record.provider_pid.or(record.host_pid))
        .filter(|pid| *pid != std::process::id())
        .map(terminate_pid)
        .unwrap_or(false);
    let stopped = remote_stopped || local_stopped;
    if stopped {
        let mut sessions = state.sessions.lock().await;
        sessions.sessions.remove(&body.session_id);
        let _ = write_daemon_registry(&state.config, &sessions);
    }
    Json(serde_json::json!({ "success": stopped }))
}

async fn control_stop(State(state): State<DaemonControlState>) -> Json<serde_json::Value> {
    if let Some(sender) = state.shutdown_tx.lock().await.take() {
        let _ = sender.send(());
    }
    Json(serde_json::json!({ "success": true }))
}

async fn daemon_post<T: serde::de::DeserializeOwned>(
    config: &Config,
    path: &str,
    body: serde_json::Value,
) -> Result<T> {
    let state = read_daemon_state(config)?.ok_or_else(|| anyhow::anyhow!("No daemon running"))?;
    let http_port = state
        .http_port
        .ok_or_else(|| anyhow::anyhow!("No daemon control API"))?;
    let response = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{http_port}{path}"))
        .json(&body)
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        bail!("daemon control request failed: {status} {body}");
    }
    Ok(serde_json::from_str(&body)?)
}

fn ensure_machine_id(config: &Config) -> Result<String> {
    let mut settings = read_settings(config)?;
    let machine_id = settings
        .machine_id
        .clone()
        .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
    if settings.machine_id.as_deref() != Some(machine_id.as_str()) {
        settings.machine_id = Some(machine_id.clone());
        write_settings(config, &settings)?;
    }
    Ok(machine_id)
}

fn machine_metadata(config: &Config) -> serde_json::Value {
    build_machine_metadata(config, &local_hostname(), detect_resume_support(config))
}

fn daemon_state_json(status: &str, started_at: u64, http_port: Option<u16>) -> serde_json::Value {
    serde_json::json!({
        "status": status,
        "pid": std::process::id(),
        "httpPort": http_port,
        "startedAt": started_at,
    })
}

fn local_hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "unknown".into())
}

fn is_pid_alive(pid: u32) -> bool {
    Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn terminate_pid(pid: u32) -> bool {
    if !is_pid_alive(pid) {
        return true;
    }
    let status = Command::new("kill")
        .arg("-TERM")
        .arg(pid.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if status.map(|value| value.success()).unwrap_or(false) {
        for _ in 0..20 {
            if !is_pid_alive(pid) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    !is_pid_alive(pid)
}

async fn server_handle_finished(handle: &tokio::task::JoinHandle<()>) {
    while !handle.is_finished() {
        sleep(Duration::from_millis(100)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::daemon_state_json;

    #[test]
    fn daemon_state_uses_stable_started_at_and_http_port() {
        let state = daemon_state_json("running", 1234, Some(8080));
        assert_eq!(state["status"], "running");
        assert_eq!(state["startedAt"], 1234);
        assert_eq!(state["httpPort"], 8080);
    }
}
