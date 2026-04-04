use anyhow::{Result as AnyhowResult, bail};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::{
    agent::core::ProviderKind,
    config::Config,
    persistence::{PersistenceError, read_settings, write_settings},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum SandboxMode {
    Disabled,
    Workspace,
}

pub fn current_mode(config: &Config) -> Result<SandboxMode, PersistenceError> {
    Ok(read_settings(config)?.sandbox_mode)
}

pub fn set_mode(config: &Config, mode: SandboxMode) -> Result<(), PersistenceError> {
    let mut settings = read_settings(config)?;
    settings.sandbox_mode = mode;
    write_settings(config, &settings)
}

pub fn disable(config: &Config) -> Result<(), PersistenceError> {
    set_mode(config, SandboxMode::Disabled)
}

#[derive(Debug, Clone, Copy)]
pub struct SandboxManager {
    mode: SandboxMode,
}

impl SandboxManager {
    pub fn new(mode: SandboxMode) -> Self {
        Self { mode }
    }

    pub fn from_config(config: &Config) -> Result<Self, PersistenceError> {
        Ok(Self {
            mode: current_mode(config)?,
        })
    }

    pub fn mode(self) -> SandboxMode {
        self.mode
    }

    pub fn apply(
        self,
        kind: ProviderKind,
        command: &mut Command,
        working_dir: &str,
    ) -> AnyhowResult<()> {
        self.ensure_supported(kind)?;
        match self.mode {
            SandboxMode::Disabled => Ok(()),
            SandboxMode::Workspace => {
                command.arg("--add-dir").arg(working_dir);
                command.env("VIBE_SANDBOX_MODE", "workspace");
                command.env("VIBE_SANDBOX_DIR", working_dir);
                Ok(())
            }
        }
    }

    pub fn ensure_supported(self, kind: ProviderKind) -> AnyhowResult<()> {
        match (self.mode, kind) {
            (SandboxMode::Workspace, ProviderKind::Claude) => Ok(()),
            (SandboxMode::Workspace, other) => bail!(
                "workspace sandbox is not supported for provider `{}` yet",
                other.as_str()
            ),
            _ => Ok(()),
        }
    }
}

pub fn describe(mode: SandboxMode) -> &'static str {
    match mode {
        SandboxMode::Disabled => "disabled",
        SandboxMode::Workspace => "workspace",
    }
}

#[cfg(test)]
mod tests {
    use tokio::process::Command;

    use crate::agent::core::ProviderKind;

    use super::{SandboxManager, SandboxMode};

    #[test]
    fn workspace_mode_rejects_providers_without_support() {
        let mut command = Command::new("echo");
        let error = SandboxManager::new(SandboxMode::Workspace)
            .apply(ProviderKind::Codex, &mut command, "/tmp")
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("workspace sandbox is not supported")
        );
    }

    #[test]
    fn workspace_mode_applies_claude_directory_flag() {
        let mut command = Command::new("claude");
        SandboxManager::new(SandboxMode::Workspace)
            .apply(ProviderKind::Claude, &mut command, "/tmp/workspace")
            .unwrap();
        let rendered = format!("{command:?}");
        assert!(rendered.contains("--add-dir"));
        assert!(rendered.contains("/tmp/workspace"));
    }
}
