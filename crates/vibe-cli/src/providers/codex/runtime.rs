use anyhow::Result;

use crate::{
    agent::core::{ProviderBackend, ProviderKind, ProviderRunRequest, ProviderRunResult},
    config::Config,
    providers::run_external::run_external_provider,
};

pub struct CodexBackend<'a> {
    config: &'a Config,
}

impl<'a> CodexBackend<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }
}

impl ProviderBackend for CodexBackend<'_> {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Codex
    }

    async fn run(&self, request: ProviderRunRequest) -> Result<ProviderRunResult> {
        let output = run_external_provider(
            &self.config.codex_bin,
            ProviderKind::Codex,
            request.provider_session_id,
            request.prompt,
            request.working_dir,
            request.resume,
            request.sandbox_mode,
            request.output_sender,
            request.started_sender,
        )
        .await?;
        Ok(ProviderRunResult { output })
    }
}
