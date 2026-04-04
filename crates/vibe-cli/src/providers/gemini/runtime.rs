use anyhow::Result;

use crate::{
    agent::core::{ProviderBackend, ProviderKind, ProviderRunRequest, ProviderRunResult},
    config::Config,
    providers::run_external::run_external_provider,
};

pub struct GeminiBackend<'a> {
    config: &'a Config,
}

impl<'a> GeminiBackend<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }
}

impl ProviderBackend for GeminiBackend<'_> {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Gemini
    }

    async fn run(&self, request: ProviderRunRequest) -> Result<ProviderRunResult> {
        let output = run_external_provider(
            &self.config.gemini_bin,
            ProviderKind::Gemini,
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
