use anyhow::Result;

use crate::{
    agent::core::{ProviderBackend, ProviderKind, ProviderRunRequest, ProviderRunResult},
    config::Config,
    providers::{
        acp::AcpBackend, claude::ClaudeBackend, codex::CodexBackend, gemini::GeminiBackend,
        openclaw::OpenclawBackend,
    },
};

pub struct ProviderRegistry<'a> {
    config: &'a Config,
}

impl<'a> ProviderRegistry<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub async fn run(
        &self,
        kind: ProviderKind,
        request: ProviderRunRequest,
    ) -> Result<ProviderRunResult> {
        match kind {
            ProviderKind::Claude => ClaudeBackend::new(self.config).run(request).await,
            ProviderKind::Codex => CodexBackend::new(self.config).run(request).await,
            ProviderKind::Gemini => GeminiBackend::new(self.config).run(request).await,
            ProviderKind::Openclaw => OpenclawBackend::new(self.config).run(request).await,
            ProviderKind::Acp => AcpBackend::new(self.config).run(request).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::PermissionsExt};

    use tempfile::TempDir;

    use crate::{
        agent::core::{ProviderKind, ProviderRunRequest},
        config::Config,
        sandbox::SandboxMode,
    };

    use super::ProviderRegistry;

    fn write_executable(path: &std::path::Path, body: &str) {
        fs::write(path, body).unwrap();
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

    #[tokio::test]
    async fn registry_dispatches_to_the_selected_provider_backend() {
        let temp_dir = TempDir::new().unwrap();
        let script = temp_dir.path().join("mock-codex.sh");
        write_executable(
            &script,
            "#!/usr/bin/env bash\nset -euo pipefail\nif [[ \"${1:-}\" == \"exec\" ]]; then\n  shift\nfi\nprintf \"registry-codex:%s\" \"$*\"\n",
        );

        let mut config = Config::from_sources(
            Some("http://127.0.0.1:3005".into()),
            Some("https://app.vibe.engineering".into()),
            Some(temp_dir.path().as_os_str().to_owned()),
            None,
        )
        .unwrap();
        config.codex_bin = script.display().to_string();

        let registry = ProviderRegistry::new(&config);
        let result = registry
            .run(
                ProviderKind::Codex,
                ProviderRunRequest {
                    provider_session_id: "provider-session".into(),
                    prompt: "hello".into(),
                    working_dir: temp_dir.path().display().to_string(),
                    resume: false,
                    sandbox_mode: SandboxMode::Disabled,
                    output_sender: None,
                    started_sender: None,
                },
            )
            .await
            .unwrap();

        assert!(result.output.contains("registry-codex:"));
        assert!(result.output.contains("--json"));
        assert!(result.output.contains("hello"));
    }
}
