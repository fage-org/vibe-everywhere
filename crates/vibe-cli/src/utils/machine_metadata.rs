use serde_json::{Value, json};
use vibe_agent::{
    config::Config as AgentConfig, credentials::read_credentials as read_agent_credentials,
};

use crate::{config::Config, modules::common::command_exists, persistence::now_ms};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CliAvailability {
    pub claude: bool,
    pub codex: bool,
    pub gemini: bool,
    pub openclaw: bool,
    pub acp: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResumeSupport {
    pub rpc_available: bool,
    pub requires_happy_agent_auth: bool,
    pub happy_agent_authenticated: bool,
}

impl ResumeSupport {
    pub fn none() -> Self {
        Self {
            rpc_available: false,
            requires_happy_agent_auth: false,
            happy_agent_authenticated: false,
        }
    }
}

pub fn detect_cli_availability(config: &Config) -> CliAvailability {
    CliAvailability {
        claude: command_exists(&config.claude_bin),
        codex: command_exists(&config.codex_bin),
        gemini: command_exists(&config.gemini_bin),
        openclaw: command_exists(&config.openclaw_bin),
        acp: command_exists(&config.acp_bin),
    }
}

pub fn detect_resume_support(config: &Config) -> ResumeSupport {
    let authenticated = read_agent_credentials(&AgentConfig {
        server_url: config.server_url.clone(),
        home_dir: config.home_dir.clone(),
        credential_path: config.home_dir.join("agent.key"),
    })
    .is_some();

    ResumeSupport {
        rpc_available: authenticated,
        requires_happy_agent_auth: true,
        happy_agent_authenticated: authenticated,
    }
}

pub fn build_machine_metadata(config: &Config, host: &str, resume_support: ResumeSupport) -> Value {
    let cli_availability = detect_cli_availability(config);
    json!({
        "host": host,
        "platform": std::env::consts::OS,
        "happyCliVersion": env!("CARGO_PKG_VERSION"),
        "homeDir": std::env::var("HOME").unwrap_or_else(|_| config.home_dir.display().to_string()),
        "happyHomeDir": config.home_dir.display().to_string(),
        "cliAvailability": {
            "claude": cli_availability.claude,
            "codex": cli_availability.codex,
            "gemini": cli_availability.gemini,
            "openclaw": cli_availability.openclaw,
            "acp": cli_availability.acp,
            "detectedAt": now_ms(),
        },
        "resumeSupport": {
            "rpcAvailable": resume_support.rpc_available,
            "requiresSameMachine": true,
            "requiresHappyAgentAuth": resume_support.requires_happy_agent_auth,
            "happyAgentAuthenticated": resume_support.happy_agent_authenticated,
            "detectedAt": now_ms(),
        }
    })
}

pub fn semantically_equal(left: &Value, right: &Value) -> bool {
    strip_detection_timestamps(left) == strip_detection_timestamps(right)
}

fn strip_detection_timestamps(value: &Value) -> Value {
    let mut normalized = value.clone();
    if let Some(cli_availability) = normalized
        .as_object_mut()
        .and_then(|root| root.get_mut("cliAvailability"))
        .and_then(Value::as_object_mut)
    {
        cli_availability.remove("detectedAt");
    }
    if let Some(resume_support) = normalized
        .as_object_mut()
        .and_then(|root| root.get_mut("resumeSupport"))
        .and_then(Value::as_object_mut)
    {
        resume_support.remove("detectedAt");
    }
    normalized
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::PermissionsExt};

    use tempfile::TempDir;

    use crate::config::Config;

    use super::{
        AgentConfig, ResumeSupport, build_machine_metadata, detect_cli_availability,
        detect_resume_support, semantically_equal,
    };

    #[test]
    fn machine_metadata_uses_happy_compatible_field_names() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::from_sources(
            Some("http://127.0.0.1:3005".into()),
            Some("https://app.vibe.engineering".into()),
            Some(temp_dir.path().as_os_str().to_owned()),
            None,
        )
        .unwrap();

        let metadata = build_machine_metadata(
            &config,
            "test-host",
            ResumeSupport {
                rpc_available: false,
                requires_happy_agent_auth: false,
                happy_agent_authenticated: false,
            },
        );

        assert_eq!(metadata["host"], "test-host");
        assert!(metadata.get("happyCliVersion").is_some());
        assert!(metadata.get("happyHomeDir").is_some());
        assert!(metadata["resumeSupport"].get("rpcAvailable").is_some());
        assert!(metadata["cliAvailability"].get("acp").is_some());
        assert!(
            metadata["resumeSupport"]
                .get("requiresHappyAgentAuth")
                .is_some()
        );
        assert!(
            metadata["resumeSupport"]
                .get("happyAgentAuthenticated")
                .is_some()
        );
        assert!(metadata.get("vibeCliVersion").is_none());
    }

    #[test]
    fn cli_availability_uses_configured_binary_paths() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("custom-codex");
        fs::write(&path, "#!/usr/bin/env bash\nexit 0\n").unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();

        let mut config = Config::from_sources(
            Some("http://127.0.0.1:3005".into()),
            Some("https://app.vibe.engineering".into()),
            Some(temp_dir.path().as_os_str().to_owned()),
            None,
        )
        .unwrap();
        config.codex_bin = path.display().to_string();

        let availability = detect_cli_availability(&config);
        assert!(availability.codex);
        assert!(!availability.acp);
    }

    #[test]
    fn resume_support_reflects_local_vibe_agent_credentials() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::from_sources(
            Some("http://127.0.0.1:3005".into()),
            Some("https://app.vibe.engineering".into()),
            Some(temp_dir.path().as_os_str().to_owned()),
            None,
        )
        .unwrap();

        let before = detect_resume_support(&config);
        assert!(!before.rpc_available);
        assert!(before.requires_happy_agent_auth);
        assert!(!before.happy_agent_authenticated);

        vibe_agent::credentials::write_credentials(
            &AgentConfig {
                server_url: config.server_url.clone(),
                home_dir: config.home_dir.clone(),
                credential_path: config.home_dir.join("agent.key"),
            },
            "token-1",
            [4u8; 32],
        )
        .unwrap();

        let after = detect_resume_support(&config);
        assert!(after.rpc_available);
        assert!(after.requires_happy_agent_auth);
        assert!(after.happy_agent_authenticated);
    }

    #[test]
    fn semantic_equality_ignores_detection_timestamps() {
        let left = serde_json::json!({
            "cliAvailability": {
                "claude": true,
                "detectedAt": 10,
            },
            "resumeSupport": {
                "rpcAvailable": false,
                "detectedAt": 20,
            }
        });
        let right = serde_json::json!({
            "cliAvailability": {
                "claude": true,
                "detectedAt": 999,
            },
            "resumeSupport": {
                "rpcAvailable": false,
                "detectedAt": 777,
            }
        });
        assert!(semantically_equal(&left, &right));
    }
}
