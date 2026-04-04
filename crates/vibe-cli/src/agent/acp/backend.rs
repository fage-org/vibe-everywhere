use anyhow::Result;

use crate::{
    agent::core::{ProviderBackend, ProviderKind, ProviderRunRequest, ProviderRunResult},
    config::Config,
    providers::run_external::run_external_provider,
};

pub struct AcpBackend<'a> {
    config: &'a Config,
}

impl<'a> AcpBackend<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }
}

impl ProviderBackend for AcpBackend<'_> {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Acp
    }

    async fn run(&self, request: ProviderRunRequest) -> Result<ProviderRunResult> {
        let output = run_external_provider(
            &self.config.acp_bin,
            ProviderKind::Acp,
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
