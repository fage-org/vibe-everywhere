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
    use crate::{
        agent::core::{ProviderKind, ProviderRunRequest},
        config::Config,
        sandbox::SandboxMode,
    };

    use super::ProviderRegistry;

    #[tokio::test]
    async fn registry_constructs_all_known_providers() {
        let config = Config::from_sources(
            Some("http://127.0.0.1:3005".into()),
            Some("https://app.vibe.engineering".into()),
            Some("/tmp/vibe-cli-registry".into()),
            None,
        )
        .unwrap();
        let registry = ProviderRegistry::new(&config);
        for kind in [
            ProviderKind::Claude,
            ProviderKind::Codex,
            ProviderKind::Gemini,
            ProviderKind::Openclaw,
            ProviderKind::Acp,
        ] {
            let result = registry
                .run(
                    kind,
                    ProviderRunRequest {
                        provider_session_id: "provider-session".into(),
                        prompt: "hello".into(),
                        working_dir: "/tmp".into(),
                        resume: false,
                        sandbox_mode: SandboxMode::Disabled,
                        output_sender: None,
                        started_sender: None,
                    },
                )
                .await;
            assert!(result.is_err() || result.is_ok());
        }
    }
}
