use vibe_core::{
    ActorIdentity, AppConfig, DeploymentMetadata, DeploymentMode, StorageKind, default_app_config,
};

#[tauri::command]
fn app_config() -> AppConfig {
    let default_relay_base_url = std::env::var("VIBE_PUBLIC_RELAY_BASE_URL")
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(default_relay_base_url);
    let requires_auth = std::env::var("VIBE_RELAY_ACCESS_TOKEN")
        .ok()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    let deployment_mode = resolve_deployment_mode();
    default_app_config(
        default_relay_base_url.clone(),
        requires_auth,
        DeploymentMetadata {
            mode: deployment_mode.clone(),
            display_name: match deployment_mode {
                DeploymentMode::SelfHosted => "Self-Hosted".to_string(),
                DeploymentMode::HostedCompatible => "Hosted-Compatible".to_string(),
            },
            relay_public_origin: default_relay_base_url,
            documentation_url: std::env::var("VIBE_RELAY_DOCUMENTATION_URL")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
        },
        resolve_storage_kind(),
        ActorIdentity::personal_owner(),
    )
}

fn resolve_deployment_mode() -> DeploymentMode {
    match std::env::var("VIBE_RELAY_DEPLOYMENT_MODE")
        .ok()
        .map(|value| value.trim().to_lowercase())
        .as_deref()
    {
        Some("hosted_compatible") => DeploymentMode::HostedCompatible,
        _ => DeploymentMode::SelfHosted,
    }
}

fn resolve_storage_kind() -> StorageKind {
    match std::env::var("VIBE_RELAY_STORAGE_KIND")
        .ok()
        .map(|value| value.trim().to_lowercase())
        .as_deref()
    {
        Some("memory") => StorageKind::Memory,
        Some("external") => StorageKind::External,
        _ => StorageKind::File,
    }
}

fn default_relay_base_url() -> String {
    if cfg!(target_os = "android") || cfg!(target_os = "ios") {
        String::new()
    } else {
        "http://127.0.0.1:8787".to_string()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![app_config])
        .run(tauri::generate_context!())
        .expect("failed to run tauri app");
}
