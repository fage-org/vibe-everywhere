use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalSessionState {
    pub id: String,
    pub provider: String,
    pub provider_session_id: String,
    pub server_session_id: String,
    pub encryption_key: Option<String>,
    pub encryption_variant: String,
    pub tag: String,
    pub working_dir: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("No local session found matching \"{0}\"")]
    SessionNotFound(String),
    #[error("Ambiguous local session \"{0}\" matches {1} sessions. Be more specific.")]
    AmbiguousSession(String, usize),
}

pub fn save_local_session(
    config: &Config,
    session: &LocalSessionState,
) -> Result<(), PersistenceError> {
    fs::create_dir_all(&config.session_state_dir)?;
    let path = session_file_path(config, &session.id);
    fs::write(path, serde_json::to_vec_pretty(session)?)?;
    Ok(())
}

pub fn remove_local_session(config: &Config, session_id: &str) -> Result<(), PersistenceError> {
    match fs::remove_file(session_file_path(config, session_id)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub fn list_local_sessions(config: &Config) -> Result<Vec<LocalSessionState>, PersistenceError> {
    if !config.session_state_dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();
    for entry in fs::read_dir(&config.session_state_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let bytes = fs::read(path)?;
        sessions.push(serde_json::from_slice::<LocalSessionState>(&bytes)?);
    }
    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(sessions)
}

pub fn resolve_local_session(
    config: &Config,
    value: &str,
) -> Result<LocalSessionState, PersistenceError> {
    let mut matches = list_local_sessions(config)?
        .into_iter()
        .filter(|session| {
            session.id.starts_with(value) || session.server_session_id.starts_with(value)
        })
        .collect::<Vec<_>>();
    match matches.len() {
        0 => Err(PersistenceError::SessionNotFound(value.into())),
        1 => Ok(matches.pop().expect("one match exists")),
        count => Err(PersistenceError::AmbiguousSession(value.into(), count)),
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_millis() as u64
}

fn session_file_path(config: &Config, id: &str) -> std::path::PathBuf {
    config.session_state_dir.join(format!("{id}.json"))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CliSettings {
    pub sandbox_mode: crate::sandbox::SandboxMode,
    pub machine_id: Option<String>,
}

impl Default for CliSettings {
    fn default() -> Self {
        Self {
            sandbox_mode: crate::sandbox::SandboxMode::Disabled,
            machine_id: None,
        }
    }
}

pub fn read_settings(config: &Config) -> Result<CliSettings, PersistenceError> {
    read_json_or_default(&config.settings_path)
}

pub fn write_settings(config: &Config, settings: &CliSettings) -> Result<(), PersistenceError> {
    write_json(&config.settings_path, settings)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonState {
    pub pid: u32,
    pub http_port: Option<u16>,
    pub started_at: u64,
    pub heartbeat_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonSessionRecord {
    pub session_id: String,
    pub provider: String,
    pub provider_session_id: String,
    pub tag: String,
    pub working_dir: String,
    pub started_at: u64,
    pub started_by: String,
    pub host_pid: Option<u32>,
    pub provider_pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DaemonRegistry {
    pub sessions: BTreeMap<String, DaemonSessionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonInstallState {
    pub launcher_path: String,
    pub executable_path: String,
    pub installed_at: u64,
}

pub fn read_daemon_registry(config: &Config) -> Result<DaemonRegistry, PersistenceError> {
    read_json_or_default(&config.daemon_registry_path)
}

pub fn write_daemon_registry(
    config: &Config,
    registry: &DaemonRegistry,
) -> Result<(), PersistenceError> {
    write_json(&config.daemon_registry_path, registry)
}

pub fn clear_daemon_registry(config: &Config) -> Result<(), PersistenceError> {
    match fs::remove_file(&config.daemon_registry_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub fn read_daemon_state(config: &Config) -> Result<Option<DaemonState>, PersistenceError> {
    if !config.daemon_state_path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&config.daemon_state_path)?;
    Ok(Some(serde_json::from_slice(&bytes)?))
}

pub fn write_daemon_state(config: &Config, state: &DaemonState) -> Result<(), PersistenceError> {
    write_json(&config.daemon_state_path, state)
}

pub fn clear_daemon_state(config: &Config) -> Result<(), PersistenceError> {
    match fs::remove_file(&config.daemon_state_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub fn read_daemon_install(
    config: &Config,
) -> Result<Option<DaemonInstallState>, PersistenceError> {
    if !config.daemon_install_path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&config.daemon_install_path)?;
    Ok(Some(serde_json::from_slice(&bytes)?))
}

pub fn write_daemon_install(
    config: &Config,
    install: &DaemonInstallState,
) -> Result<(), PersistenceError> {
    write_json(&config.daemon_install_path, install)
}

pub fn clear_daemon_install(config: &Config) -> Result<(), PersistenceError> {
    match fs::remove_file(&config.daemon_install_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), PersistenceError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn read_json_or_default<T>(path: &Path) -> Result<T, PersistenceError>
where
    T: serde::de::DeserializeOwned + Default,
{
    if !path.exists() {
        return Ok(T::default());
    }
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::{config::Config, sandbox::SandboxMode};

    use super::{
        CliSettings, DaemonInstallState, DaemonRegistry, DaemonSessionRecord, DaemonState,
        LocalSessionState, clear_daemon_install, clear_daemon_registry, clear_daemon_state,
        list_local_sessions, now_ms, read_daemon_install, read_daemon_registry, read_daemon_state,
        read_settings, remove_local_session, resolve_local_session, save_local_session,
        write_daemon_install, write_daemon_registry, write_daemon_state, write_settings,
    };

    fn test_config() -> (TempDir, Config) {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::from_sources(
            Some("http://127.0.0.1:3005".into()),
            Some("https://app.vibe.engineering".into()),
            Some(temp_dir.path().as_os_str().to_owned()),
            None,
        )
        .unwrap();
        (temp_dir, config)
    }

    #[test]
    fn local_session_round_trip_and_prefix_resolution_work() {
        let (_temp_dir, config) = test_config();
        let session = LocalSessionState {
            id: "session-abc".into(),
            provider: "claude".into(),
            provider_session_id: "provider-1".into(),
            server_session_id: "server-1".into(),
            encryption_key: Some("Zm9v".into()),
            encryption_variant: "dataKey".into(),
            tag: "demo".into(),
            working_dir: "/tmp/demo".into(),
            created_at: now_ms(),
            updated_at: now_ms(),
        };

        save_local_session(&config, &session).unwrap();
        let sessions = list_local_sessions(&config).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(
            resolve_local_session(&config, "session-a").unwrap(),
            session
        );
        remove_local_session(&config, &session.id).unwrap();
        assert!(list_local_sessions(&config).unwrap().is_empty());
    }

    #[test]
    fn settings_round_trip() {
        let (_temp_dir, config) = test_config();
        write_settings(
            &config,
            &CliSettings {
                sandbox_mode: SandboxMode::Workspace,
                machine_id: Some("machine-1".into()),
            },
        )
        .unwrap();
        assert_eq!(
            read_settings(&config).unwrap().sandbox_mode,
            SandboxMode::Workspace
        );
        assert_eq!(
            read_settings(&config).unwrap().machine_id.as_deref(),
            Some("machine-1")
        );
    }

    #[test]
    fn daemon_state_round_trip_and_clear() {
        let (_temp_dir, config) = test_config();
        let state = DaemonState {
            pid: 42,
            http_port: Some(3007),
            started_at: 1,
            heartbeat_at: 2,
        };
        write_daemon_state(&config, &state).unwrap();
        assert_eq!(read_daemon_state(&config).unwrap(), Some(state));
        clear_daemon_state(&config).unwrap();
        assert_eq!(read_daemon_state(&config).unwrap(), None);
    }

    #[test]
    fn daemon_registry_round_trip_and_clear() {
        let (_temp_dir, config) = test_config();
        let mut registry = DaemonRegistry::default();
        registry.sessions.insert(
            "session-1".into(),
            DaemonSessionRecord {
                session_id: "session-1".into(),
                provider: "claude".into(),
                provider_session_id: "provider-1".into(),
                tag: "demo".into(),
                working_dir: "/tmp/demo".into(),
                started_at: 1,
                started_by: "terminal".into(),
                host_pid: Some(123),
                provider_pid: Some(456),
            },
        );
        write_daemon_registry(&config, &registry).unwrap();
        assert_eq!(read_daemon_registry(&config).unwrap(), registry);
        clear_daemon_registry(&config).unwrap();
        assert!(read_daemon_registry(&config).unwrap().sessions.is_empty());
    }

    #[test]
    fn daemon_install_round_trip_and_clear() {
        let (_temp_dir, config) = test_config();
        let install = DaemonInstallState {
            launcher_path: "/tmp/vibe-daemon".into(),
            executable_path: "/tmp/vibe".into(),
            installed_at: 42,
        };
        write_daemon_install(&config, &install).unwrap();
        assert_eq!(read_daemon_install(&config).unwrap(), Some(install));
        clear_daemon_install(&config).unwrap();
        assert_eq!(read_daemon_install(&config).unwrap(), None);
    }
}
