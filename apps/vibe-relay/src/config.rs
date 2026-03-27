use std::path::PathBuf;

#[derive(Clone, Debug)]
pub(crate) struct RelayConfig {
    pub(crate) public_base_url: String,
    pub(crate) access_token: Option<String>,
    pub(crate) state_file: PathBuf,
    pub(crate) forward_host: String,
    pub(crate) forward_bind_host: String,
    pub(crate) forward_port_start: u16,
    pub(crate) forward_port_end: u16,
    pub(crate) shell_bridge_port: u16,
    pub(crate) port_forward_bridge_port: u16,
    pub(crate) task_bridge_port: u16,
}

impl RelayConfig {
    pub(crate) fn from_env(bind_host: &str, bind_port: &str) -> Self {
        Self {
            public_base_url: resolve_public_base_url(bind_host, bind_port),
            access_token: std::env::var("VIBE_RELAY_ACCESS_TOKEN")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            state_file: resolve_state_file(),
            forward_host: resolve_forward_host(bind_host),
            forward_bind_host: resolve_forward_bind_host(bind_host),
            forward_port_start: resolve_forward_port_start(),
            forward_port_end: resolve_forward_port_end(),
            shell_bridge_port: resolve_shell_bridge_port(),
            port_forward_bridge_port: resolve_port_forward_bridge_port(),
            task_bridge_port: resolve_task_bridge_port(),
        }
    }
}

fn resolve_public_base_url(bind_host: &str, bind_port: &str) -> String {
    if let Some(base_url) = std::env::var("VIBE_PUBLIC_RELAY_BASE_URL")
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty())
    {
        return base_url;
    }

    let host = match bind_host {
        "0.0.0.0" | "::" => "127.0.0.1",
        value => value,
    };
    format!("http://{host}:{bind_port}")
}

fn resolve_forward_host(bind_host: &str) -> String {
    if let Some(host) = std::env::var("VIBE_RELAY_FORWARD_HOST")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return host;
    }

    match bind_host {
        "0.0.0.0" | "::" => "127.0.0.1".to_string(),
        value => value.to_string(),
    }
}

fn resolve_forward_bind_host(bind_host: &str) -> String {
    std::env::var("VIBE_RELAY_FORWARD_BIND_HOST")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| bind_host.to_string())
}

fn resolve_forward_port_start() -> u16 {
    resolve_forward_port_value("VIBE_RELAY_FORWARD_PORT_START", 39_000)
}

fn resolve_forward_port_end() -> u16 {
    resolve_forward_port_value("VIBE_RELAY_FORWARD_PORT_END", 39_999)
}

fn resolve_shell_bridge_port() -> u16 {
    std::env::var("VIBE_AGENT_SHELL_BRIDGE_PORT")
        .ok()
        .and_then(|value| value.trim().parse::<u16>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(crate::DEFAULT_SHELL_BRIDGE_PORT)
}

fn resolve_port_forward_bridge_port() -> u16 {
    std::env::var("VIBE_AGENT_PORT_FORWARD_BRIDGE_PORT")
        .ok()
        .and_then(|value| value.trim().parse::<u16>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(crate::DEFAULT_PORT_FORWARD_BRIDGE_PORT)
}

fn resolve_task_bridge_port() -> u16 {
    std::env::var("VIBE_AGENT_TASK_BRIDGE_PORT")
        .ok()
        .and_then(|value| value.trim().parse::<u16>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(crate::DEFAULT_TASK_BRIDGE_PORT)
}

fn resolve_forward_port_value(name: &str, default: u16) -> u16 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u16>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn resolve_state_file() -> PathBuf {
    if let Some(path) = std::env::var("VIBE_RELAY_STATE_FILE")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return PathBuf::from(path);
    }

    default_state_file()
}

fn default_state_file() -> PathBuf {
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));
    let state_file = base.join(".vibe-everywhere").join("relay-state.json");
    if state_file.exists() {
        return state_file;
    }

    let legacy_state_file = base.join(".vibe-remote").join("relay-state.json");
    if legacy_state_file.exists() {
        return legacy_state_file;
    }

    state_file
}
