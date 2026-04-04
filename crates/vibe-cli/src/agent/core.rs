use anyhow::Result;
use tokio::sync::mpsc;

use crate::sandbox::SandboxMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Claude,
    Codex,
    Gemini,
    Openclaw,
    Acp,
}

impl ProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
            Self::Openclaw => "openclaw",
            Self::Acp => "acp",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderProcessInfo {
    pub provider_pid: u32,
}

#[derive(Debug)]
pub struct ProviderRunRequest {
    pub provider_session_id: String,
    pub prompt: String,
    pub working_dir: String,
    pub resume: bool,
    pub sandbox_mode: SandboxMode,
    pub output_sender: Option<mpsc::Sender<String>>,
    pub started_sender: Option<mpsc::Sender<ProviderProcessInfo>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRunResult {
    pub output: String,
}

pub trait ProviderBackend: Send + Sync {
    fn kind(&self) -> ProviderKind;
    fn run(
        &self,
        request: ProviderRunRequest,
    ) -> impl std::future::Future<Output = Result<ProviderRunResult>> + Send;
}
