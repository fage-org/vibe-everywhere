use anyhow::{Context, Result, bail};
use cidr::Ipv4Inet;
use easytier::{
    common::config::{
        ConfigFileControl, ConfigLoader, NetworkIdentity, PeerConfig, TomlConfigLoader,
        gen_default_flags,
    },
    launcher::{MyNodeInfo, NetworkInstance},
};
use std::{env, net::Ipv4Addr, sync::Arc, time::Duration};
use tokio::{
    sync::{RwLock, watch},
    task::JoinHandle,
};
use url::Url;
use uuid::Uuid;
use vibe_core::{OverlayMode, OverlayNetworkStatus, OverlayState};

const DEFAULT_EASYTIER_RESTART_DELAY_SECS: u64 = 5;
const DEFAULT_EASYTIER_STATUS_POLL_SECS: u64 = 3;
const DEFAULT_LISTENER_PORT: u16 = 11010;

#[derive(Debug, Clone)]
pub struct AgentEasyTierOptions {
    pub device_id: String,
    pub device_name: String,
    pub instance_name: String,
    pub network_name: Option<String>,
    pub network_secret: Option<String>,
    pub peers: Vec<String>,
    pub node_ip: Option<String>,
    pub no_listener: bool,
    pub listeners: Vec<String>,
}

pub struct ManagedEasyTierRuntime {
    shutdown_tx: watch::Sender<bool>,
    task: JoinHandle<()>,
}

impl ManagedEasyTierRuntime {
    pub async fn shutdown(self) {
        let _ = self.shutdown_tx.send(true);
        let _ = self.task.await;
    }
}

impl AgentEasyTierOptions {
    pub fn from_inputs(
        device_id: &str,
        device_name: &str,
        network_name: Option<String>,
        bootstrap_urls: Option<String>,
        node_ip: Option<String>,
    ) -> Self {
        Self {
            device_id: device_id.to_string(),
            device_name: device_name.to_string(),
            instance_name: env::var("VIBE_EASYTIER_INSTANCE_NAME")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| format!("vibe-agent-{}", short_device_id(device_id))),
            network_name: network_name
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            network_secret: env::var("VIBE_EASYTIER_NETWORK_SECRET")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            peers: parse_list_values(
                &bootstrap_urls
                    .unwrap_or_else(|| env::var("VIBE_EASYTIER_BOOTSTRAP_URL").unwrap_or_default()),
            ),
            node_ip: node_ip
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            no_listener: parse_bool_env("VIBE_EASYTIER_NO_LISTENER").unwrap_or(true),
            listeners: parse_list_values(&env::var("VIBE_EASYTIER_LISTENERS").unwrap_or_default()),
        }
    }

    pub fn enabled(&self) -> bool {
        self.network_name.is_some()
    }
}

pub fn initial_overlay_status(options: &AgentEasyTierOptions) -> OverlayNetworkStatus {
    if !options.enabled() {
        return OverlayNetworkStatus::default();
    }

    OverlayNetworkStatus {
        mode: OverlayMode::EasyTierEmbedded,
        state: OverlayState::Degraded,
        network_name: options.network_name.clone(),
        node_ip: options
            .node_ip
            .as_deref()
            .map(strip_cidr_suffix)
            .map(str::to_string),
        relay_url: join_peers(&options.peers),
        binary_path: None,
        last_error: if options.peers.is_empty() {
            Some(
                "EasyTier embedded node is configured without bootstrap peers; add a peer or listener so other nodes can reach it"
                    .to_string(),
            )
        } else {
            None
        },
    }
}

pub fn start_managed_agent_easytier(
    options: AgentEasyTierOptions,
    overlay: Arc<RwLock<OverlayNetworkStatus>>,
) -> Option<ManagedEasyTierRuntime> {
    if !options.enabled() {
        return None;
    }

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let task = tokio::spawn(async move {
        run_supervisor(options, overlay, shutdown_rx).await;
    });

    Some(ManagedEasyTierRuntime { task, shutdown_tx })
}

async fn run_supervisor(
    options: AgentEasyTierOptions,
    overlay: Arc<RwLock<OverlayNetworkStatus>>,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    loop {
        if *shutdown_rx.borrow() {
            break;
        }

        match start_agent_instance(&options) {
            Ok(instance) => {
                eprintln!(
                    "[easytier-agent] embedded instance started inst={} network={}",
                    options.instance_name,
                    options.network_name.as_deref().unwrap_or("default")
                );
                if let Err(error) =
                    monitor_instance(&instance, &options, &overlay, &mut shutdown_rx).await
                {
                    eprintln!("[easytier-agent] embedded instance stopped: {error:#}");
                    update_overlay(
                        &overlay,
                        OverlayState::Unavailable,
                        options
                            .node_ip
                            .as_deref()
                            .map(strip_cidr_suffix)
                            .map(str::to_string),
                        Some(error.to_string()),
                        options.network_name.clone(),
                        options.peers.clone(),
                    )
                    .await;
                }
                drop(instance);
            }
            Err(error) => {
                eprintln!("[easytier-agent] failed to start embedded instance: {error:#}");
                update_overlay(
                    &overlay,
                    OverlayState::Unavailable,
                    options
                        .node_ip
                        .as_deref()
                        .map(strip_cidr_suffix)
                        .map(str::to_string),
                    Some(error.to_string()),
                    options.network_name.clone(),
                    options.peers.clone(),
                )
                .await;
            }
        }

        tokio::select! {
            _ = shutdown_rx.changed() => break,
            _ = tokio::time::sleep(Duration::from_secs(easytier_restart_delay_secs())) => {}
        }
    }
}

fn start_agent_instance(options: &AgentEasyTierOptions) -> Result<NetworkInstance> {
    let config = build_agent_config(options)?;
    let mut instance = NetworkInstance::new(config, ConfigFileControl::STATIC_CONFIG);
    instance
        .start()
        .context("failed to start embedded EasyTier agent instance")?;
    Ok(instance)
}

async fn monitor_instance(
    instance: &NetworkInstance,
    options: &AgentEasyTierOptions,
    overlay: &Arc<RwLock<OverlayNetworkStatus>>,
    shutdown_rx: &mut watch::Receiver<bool>,
) -> Result<()> {
    let mut poll = tokio::time::interval(Duration::from_secs(easytier_status_poll_secs()));

    loop {
        tokio::select! {
            _ = shutdown_rx.changed() => break,
            _ = poll.tick() => {
                if !instance.is_easytier_running() {
                    let error = instance
                        .get_latest_error_msg()
                        .unwrap_or_else(|| "embedded EasyTier agent instance stopped unexpectedly".to_string());
                    update_overlay(
                        overlay,
                        OverlayState::Unavailable,
                        options.node_ip.as_deref().map(strip_cidr_suffix).map(str::to_string),
                        Some(error),
                        options.network_name.clone(),
                        options.peers.clone(),
                    )
                    .await;
                    bail!("embedded EasyTier agent instance stopped");
                }

                sync_overlay(instance, options, overlay).await;
            }
        }
    }

    Ok(())
}

async fn sync_overlay(
    instance: &NetworkInstance,
    options: &AgentEasyTierOptions,
    overlay: &Arc<RwLock<OverlayNetworkStatus>>,
) {
    let configured_node_ip = options
        .node_ip
        .as_deref()
        .map(strip_cidr_suffix)
        .map(str::to_string);
    match instance.get_running_info().await {
        Ok(info) => {
            let node_ip =
                extract_overlay_node_ip(info.my_node_info.as_ref()).or(configured_node_ip);
            let state = running_overlay_state(options);
            update_overlay(
                overlay,
                state,
                node_ip,
                info.error_msg.clone(),
                options.network_name.clone(),
                options.peers.clone(),
            )
            .await;
        }
        Err(error) => {
            update_overlay(
                overlay,
                OverlayState::Degraded,
                configured_node_ip,
                Some(format!("embedded EasyTier agent RPC not ready: {error}")),
                options.network_name.clone(),
                options.peers.clone(),
            )
            .await;
        }
    }
}

fn build_agent_config(options: &AgentEasyTierOptions) -> Result<TomlConfigLoader> {
    let cfg = TomlConfigLoader::default();
    cfg.set_id(Uuid::parse_str(&options.device_id).unwrap_or_else(|_| Uuid::new_v4()));
    cfg.set_hostname(Some(options.device_name.clone()));
    cfg.set_inst_name(options.instance_name.clone());
    cfg.set_dhcp(options.node_ip.is_none());
    cfg.set_network_identity(NetworkIdentity::new(
        options
            .network_name
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        options.network_secret.clone().unwrap_or_default(),
    ));
    if let Some(node_ip) = &options.node_ip {
        cfg.set_ipv4(Some(parse_ipv4_inet(node_ip)?));
    }
    cfg.set_peers(parse_peer_configs(&options.peers)?);
    cfg.set_listeners(resolve_agent_listener_urls(options)?);

    let mut flags = gen_default_flags();
    flags.private_mode = options.network_name.is_some();
    flags.multi_thread = true;
    cfg.set_flags(flags);

    Ok(cfg)
}

fn parse_peer_configs(peers: &[String]) -> Result<Vec<PeerConfig>> {
    peers
        .iter()
        .map(|peer| {
            Ok(PeerConfig {
                uri: peer
                    .parse::<Url>()
                    .with_context(|| format!("failed to parse EasyTier peer URL: {peer}"))?,
                peer_public_key: None,
            })
        })
        .collect()
}

fn resolve_agent_listener_urls(options: &AgentEasyTierOptions) -> Result<Vec<Url>> {
    parse_listener_urls(options.no_listener, &options.listeners)
}

fn parse_listener_urls(no_listener: bool, listeners: &[String]) -> Result<Vec<Url>> {
    if no_listener {
        return Ok(vec![]);
    }

    let source = if listeners.is_empty() {
        vec![DEFAULT_LISTENER_PORT.to_string()]
    } else {
        listeners.to_vec()
    };

    let mut resolved = Vec::new();
    if source.len() == 1 {
        if let Ok(port) = source[0].parse::<u16>() {
            resolved.push(format!("tcp://0.0.0.0:{port}").parse()?);
            resolved.push(format!("udp://0.0.0.0:{port}").parse()?);
            return Ok(resolved);
        }
    }

    for raw in source {
        let value = raw.trim();
        if value.is_empty() {
            continue;
        }
        if value.contains("://") {
            resolved.push(
                value
                    .parse::<Url>()
                    .with_context(|| format!("failed to parse EasyTier listener URL: {value}"))?,
            );
            continue;
        }

        let parts: Vec<&str> = value.split(':').collect();
        let (proto, port) = match parts.as_slice() {
            [proto] => (*proto, DEFAULT_LISTENER_PORT),
            [proto, port] => (
                *proto,
                port.parse::<u16>().with_context(|| {
                    format!("failed to parse EasyTier listener port in {value}")
                })?,
            ),
            _ => bail!("unsupported EasyTier listener format: {value}"),
        };

        match proto {
            "tcp" | "udp" => {
                resolved.push(format!("{proto}://0.0.0.0:{port}").parse()?);
            }
            _ => {
                bail!(
                    "unsupported EasyTier listener shorthand: {value}; use tcp/udp shorthand or a full URL"
                );
            }
        }
    }

    Ok(resolved)
}

fn parse_ipv4_inet(raw: &str) -> Result<Ipv4Inet> {
    let value = raw.trim();
    if value.is_empty() {
        bail!("EasyTier node IP cannot be empty");
    }

    let normalized = if value.contains('/') {
        value.to_string()
    } else {
        format!("{value}/24")
    };

    normalized
        .parse::<Ipv4Inet>()
        .with_context(|| format!("failed to parse EasyTier node IP: {value}"))
}

// Once the embedded runtime RPC is readable again, bridge-specific availability is verified
// separately by the relay and smoke harness.
fn running_overlay_state(options: &AgentEasyTierOptions) -> OverlayState {
    if options.peers.is_empty() {
        OverlayState::Degraded
    } else {
        OverlayState::Connected
    }
}

async fn update_overlay(
    overlay: &Arc<RwLock<OverlayNetworkStatus>>,
    state: OverlayState,
    node_ip: Option<String>,
    last_error: Option<String>,
    network_name: Option<String>,
    peers: Vec<String>,
) {
    let mut guard = overlay.write().await;
    guard.mode = OverlayMode::EasyTierEmbedded;
    guard.state = state;
    guard.network_name = network_name;
    guard.node_ip = node_ip;
    guard.relay_url = join_peers(&peers);
    guard.binary_path = None;
    guard.last_error = last_error;
}

fn join_peers(peers: &[String]) -> Option<String> {
    if peers.is_empty() {
        None
    } else {
        Some(peers.join(","))
    }
}

fn extract_overlay_node_ip(node: Option<&MyNodeInfo>) -> Option<String> {
    let inet = node?.virtual_ipv4.as_ref()?;
    let address = inet.address.as_ref()?;
    Some(Ipv4Addr::from(address.addr).to_string())
}

fn strip_cidr_suffix(value: &str) -> &str {
    value.split('/').next().unwrap_or(value)
}

fn parse_list_values(raw: &str) -> Vec<String> {
    raw.split(|ch: char| ch == ',' || ch.is_ascii_whitespace())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn parse_bool_env(name: &str) -> Option<bool> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .and_then(|value| match value.as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        })
}

fn parse_u64_env(name: &str) -> Option<u64> {
    env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
}

fn easytier_restart_delay_secs() -> u64 {
    parse_u64_env("VIBE_EASYTIER_RESTART_DELAY_SECS").unwrap_or(DEFAULT_EASYTIER_RESTART_DELAY_SECS)
}

fn easytier_status_poll_secs() -> u64 {
    parse_u64_env("VIBE_EASYTIER_STATUS_POLL_SECS").unwrap_or(DEFAULT_EASYTIER_STATUS_POLL_SECS)
}

fn short_device_id(device_id: &str) -> &str {
    device_id.get(..8).unwrap_or(device_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_agent_config_preserves_private_network_settings() {
        let options = AgentEasyTierOptions {
            device_id: Uuid::new_v4().to_string(),
            device_name: "Workstation".to_string(),
            instance_name: "vibe-agent-1234".to_string(),
            network_name: Some("personal-net".to_string()),
            network_secret: Some("top-secret".to_string()),
            peers: vec![
                "tcp://relay.example.com:11010".to_string(),
                "udp://relay.example.com:11010".to_string(),
            ],
            node_ip: Some("10.11.12.13".to_string()),
            no_listener: true,
            listeners: vec![],
        };

        let cfg = build_agent_config(&options).expect("agent config should build");
        assert_eq!(cfg.get_inst_name(), "vibe-agent-1234");
        assert_eq!(cfg.get_hostname(), "Workstation");
        assert!(!cfg.get_dhcp());
        assert_eq!(cfg.get_listeners().unwrap(), Vec::<Url>::new());
        assert_eq!(cfg.get_peers().len(), 2);
        assert!(cfg.get_flags().private_mode);
        assert_eq!(cfg.get_ipv4().unwrap().address().to_string(), "10.11.12.13");
    }

    #[test]
    fn agent_listener_defaults_expand_single_port() {
        let listeners = parse_listener_urls(false, &["11010".to_string()]).expect("listeners");
        assert_eq!(listeners[0].as_str(), "tcp://0.0.0.0:11010");
        assert_eq!(listeners[1].as_str(), "udp://0.0.0.0:11010");
    }

    #[test]
    fn parse_list_values_supports_commas_and_whitespace() {
        let values = parse_list_values(
            "tcp://a:1, ws://b:2
quic://c:3",
        );
        assert_eq!(values, vec!["tcp://a:1", "ws://b:2", "quic://c:3"]);
    }

    #[test]
    fn running_overlay_state_requires_bootstrap_peer_configuration() {
        let disconnected = AgentEasyTierOptions {
            device_id: Uuid::new_v4().to_string(),
            device_name: "Workstation".to_string(),
            instance_name: "vibe-agent-1234".to_string(),
            network_name: Some("personal-net".to_string()),
            network_secret: Some("top-secret".to_string()),
            peers: vec![],
            node_ip: Some("10.11.12.13".to_string()),
            no_listener: true,
            listeners: vec![],
        };
        let connected = AgentEasyTierOptions {
            peers: vec!["tcp://relay.example.com:11010".to_string()],
            ..disconnected.clone()
        };

        assert_eq!(running_overlay_state(&disconnected), OverlayState::Degraded);
        assert_eq!(running_overlay_state(&connected), OverlayState::Connected);
    }
}
