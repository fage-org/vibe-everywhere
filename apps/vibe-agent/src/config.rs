use anyhow::{Context, Result, bail};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use url::Url;
use vibe_core::DevicePlatform;

pub(crate) fn base_metadata(working_root: &Path) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("arch".to_string(), std::env::consts::ARCH.to_string()),
        (
            "os".to_string(),
            DevicePlatform::current().label().to_string(),
        ),
        (
            "workingRoot".to_string(),
            working_root.to_string_lossy().to_string(),
        ),
    ])
}

pub(crate) fn normalize_base_url(url: &str) -> String {
    url.trim_end_matches('/').to_string()
}

pub(crate) fn default_agent_identity_path(working_root: &Path) -> PathBuf {
    working_root.join(".vibe-agent").join("identity.json")
}

pub(crate) fn build_relay_websocket_url(
    relay_url: &str,
    path: &str,
    device_id: &str,
    access_token: Option<&str>,
) -> Result<String> {
    let mut url =
        Url::parse(relay_url).with_context(|| format!("invalid relay URL: {relay_url}"))?;
    let scheme = match url.scheme() {
        "http" => "ws",
        "https" => "wss",
        "ws" => "ws",
        "wss" => "wss",
        other => bail!("unsupported relay URL scheme: {other}"),
    };
    url.set_scheme(scheme)
        .map_err(|_| anyhow::anyhow!("failed to rewrite relay URL scheme"))?;

    let normalized_path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    let base_path = url.path().trim_end_matches('/');
    let full_path = if base_path.is_empty() || base_path == "/" {
        normalized_path
    } else {
        format!("{base_path}{normalized_path}")
    };
    url.set_path(&full_path);
    url.set_query(None);

    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("deviceId", device_id);
        if let Some(token) = access_token.filter(|value| !value.is_empty()) {
            pairs.append_pair("access_token", token);
        }
    }

    Ok(url.to_string())
}

pub(crate) fn resolve_working_root(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        path.canonicalize()
            .with_context(|| format!("failed to canonicalize {}", path.display()))
    } else {
        bail!("working root does not exist: {}", path.display());
    }
}

pub(crate) fn resolve_task_cwd(working_root: &Path, cwd: Option<&str>) -> PathBuf {
    match cwd {
        Some(value) if !value.trim().is_empty() => {
            let path = PathBuf::from(value);
            if path.is_absolute() {
                path
            } else {
                working_root.join(path)
            }
        }
        _ => working_root.to_path_buf(),
    }
}

pub(crate) fn ensure_task_cwd(path: &Path) -> Result<PathBuf> {
    if !path.exists() {
        bail!("task cwd does not exist: {}", path.display());
    }
    if !path.is_dir() {
        bail!("task cwd is not a directory: {}", path.display());
    }
    path.canonicalize()
        .with_context(|| format!("failed to canonicalize task cwd {}", path.display()))
}

pub(crate) fn default_device_name() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| format!("{}-node", DevicePlatform::current().label().to_lowercase()))
}
