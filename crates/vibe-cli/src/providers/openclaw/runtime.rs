use anyhow::Result;

use crate::{
    agent::core::{ProviderBackend, ProviderKind, ProviderRunRequest, ProviderRunResult},
    config::Config,
    providers::run_external::run_external_provider,
};

pub struct OpenclawBackend<'a> {
    config: &'a Config,
}

impl<'a> OpenclawBackend<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }
}

impl ProviderBackend for OpenclawBackend<'_> {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Openclaw
    }

    async fn run(&self, request: ProviderRunRequest) -> Result<ProviderRunResult> {
        let output = run_external_provider(
            &self.config.openclaw_bin,
            ProviderKind::Openclaw,
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
