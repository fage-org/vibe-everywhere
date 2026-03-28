use anyhow::{Context, Result, bail};
use easytier::{
    common::config::{
        ConfigFileControl, ConfigLoader, NetworkIdentity, PeerConfig, TomlConfigLoader,
        gen_default_flags,
    },
    launcher::NetworkInstance,
};
use std::{env, time::Duration};
use tokio::{sync::watch, task::JoinHandle};
use url::Url;
use uuid::Uuid;

const DEFAULT_EASYTIER_RESTART_DELAY_SECS: u64 = 5;
const DEFAULT_EASYTIER_STATUS_POLL_SECS: u64 = 3;
const DEFAULT_LISTENER_PORT: u16 = 11010;

#[derive(Debug, Clone)]
pub struct RelayEasyTierOptions {
    pub enabled: bool,
    pub instance_name: String,
    pub hostname: String,
    pub network_name: Option<String>,
    pub network_secret: Option<String>,
    pub peers: Vec<String>,
    pub listeners: Vec<String>,
    pub private_mode: bool,
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

impl RelayEasyTierOptions {
    pub fn from_env() -> Self {
        let network_name = env::var("VIBE_EASYTIER_NETWORK_NAME")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let explicitly_enabled = parse_bool_env("VIBE_EASYTIER_RELAY_ENABLED").unwrap_or(false);
        Self {
            enabled: explicitly_enabled || network_name.is_some(),
            instance_name: env::var("VIBE_EASYTIER_INSTANCE_NAME")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "vibe-relay".to_string()),
            hostname: env::var("VIBE_EASYTIER_HOSTNAME")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "vibe-relay".to_string()),
            network_name: network_name.clone(),
            network_secret: env::var("VIBE_EASYTIER_NETWORK_SECRET")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            peers: parse_list_values(&env::var("VIBE_EASYTIER_BOOTSTRAP_URL").unwrap_or_default()),
            listeners: parse_list_values(&env::var("VIBE_EASYTIER_LISTENERS").unwrap_or_default()),
            private_mode: parse_bool_env("VIBE_EASYTIER_PRIVATE_MODE")
                .unwrap_or(network_name.is_some()),
        }
    }
}

pub fn start_managed_relay_easytier(
    options: RelayEasyTierOptions,
) -> Option<ManagedEasyTierRuntime> {
    if !options.enabled {
        return None;
    }

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let task = tokio::spawn(async move {
        run_supervisor(options, shutdown_rx).await;
    });

    Some(ManagedEasyTierRuntime { task, shutdown_tx })
}

async fn run_supervisor(options: RelayEasyTierOptions, mut shutdown_rx: watch::Receiver<bool>) {
    loop {
        if *shutdown_rx.borrow() {
            break;
        }

        match start_relay_instance(&options) {
            Ok(instance) => {
                eprintln!(
                    "[easytier-relay] embedded instance started inst={} network={}",
                    options.instance_name,
                    options.network_name.as_deref().unwrap_or("default")
                );
                if let Err(error) = monitor_instance(&instance, &mut shutdown_rx).await {
                    eprintln!("[easytier-relay] embedded instance stopped: {error:#}");
                }
                drop(instance);
            }
            Err(error) => {
                eprintln!("[easytier-relay] failed to start embedded instance: {error:#}");
            }
        }

        tokio::select! {
            _ = shutdown_rx.changed() => break,
            _ = tokio::time::sleep(Duration::from_secs(easytier_restart_delay_secs())) => {}
        }
    }
}

fn start_relay_instance(options: &RelayEasyTierOptions) -> Result<NetworkInstance> {
    let config = build_relay_config(options)?;
    let mut instance = NetworkInstance::new(config, ConfigFileControl::STATIC_CONFIG);
    instance
        .start()
        .context("failed to start embedded EasyTier relay instance")?;
    Ok(instance)
}

async fn monitor_instance(
    instance: &NetworkInstance,
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
                        .unwrap_or_else(|| "embedded EasyTier relay instance stopped unexpectedly".to_string());
                    bail!(error);
                }
            }
        }
    }

    Ok(())
}

fn build_relay_config(options: &RelayEasyTierOptions) -> Result<TomlConfigLoader> {
    let cfg = TomlConfigLoader::default();
    cfg.set_id(Uuid::new_v4());
    cfg.set_hostname(Some(options.hostname.clone()));
    cfg.set_inst_name(options.instance_name.clone());
    cfg.set_dhcp(true);
    cfg.set_network_identity(NetworkIdentity::new(
        options
            .network_name
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        options.network_secret.clone().unwrap_or_default(),
    ));
    cfg.set_peers(parse_peer_configs(&options.peers)?);
    cfg.set_listeners(resolve_relay_listener_urls(&options.listeners)?);

    let mut flags = gen_default_flags();
    flags.private_mode = options.private_mode && options.network_name.is_some();
    flags.multi_thread = true;
    cfg.set_flags(flags);

    Ok(cfg)
}

fn resolve_relay_listener_urls(listeners: &[String]) -> Result<Vec<Url>> {
    if listeners.is_empty() {
        parse_listener_urls(&[DEFAULT_LISTENER_PORT.to_string()])
    } else {
        parse_listener_urls(listeners)
    }
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

fn parse_listener_urls(listeners: &[String]) -> Result<Vec<Url>> {
    let mut resolved = Vec::new();
    if listeners.len() == 1 {
        if let Ok(port) = listeners[0].parse::<u16>() {
            resolved.push(format!("tcp://0.0.0.0:{port}").parse()?);
            resolved.push(format!("udp://0.0.0.0:{port}").parse()?);
            return Ok(resolved);
        }
    }

    for raw in listeners {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_relay_config_supports_private_network_mode() {
        let options = RelayEasyTierOptions {
            enabled: true,
            instance_name: "vibe-relay".to_string(),
            hostname: "relay".to_string(),
            network_name: Some("personal-net".to_string()),
            network_secret: Some("secret".to_string()),
            peers: vec!["tcp://seed.example.com:11010".to_string()],
            listeners: vec!["tcp://0.0.0.0:11010".to_string()],
            private_mode: true,
        };

        let cfg = build_relay_config(&options).expect("relay config should build");
        assert_eq!(cfg.get_inst_name(), "vibe-relay");
        assert_eq!(cfg.get_hostname(), "relay");
        assert_eq!(cfg.get_peers().len(), 1);
        assert_eq!(cfg.get_listeners().unwrap().len(), 1);
        assert!(cfg.get_flags().private_mode);
    }

    #[test]
    fn relay_config_defaults_listener_port_when_unspecified() {
        let urls = resolve_relay_listener_urls(&[]).expect("default listeners");
        assert_eq!(urls[0].as_str(), "tcp://0.0.0.0:11010");
        assert_eq!(urls[1].as_str(), "udp://0.0.0.0:11010");
    }
}
