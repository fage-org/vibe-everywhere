use tempfile::TempDir;
use vibe_cli::{
    agent::{
        core::{ProviderBackend, ProviderRunRequest},
        registry::ProviderRegistry,
    },
    config::Config,
    providers::{
        acp::AcpBackend, codex::CodexBackend, gemini::GeminiBackend, openclaw::OpenclawBackend,
    },
    sandbox::SandboxMode,
};

mod fixtures;

use fixtures::write_executable;

fn make_script(temp_dir: &TempDir, name: &str, body: &str) -> String {
    let path = temp_dir.path().join(name);
    write_executable(&path, body);
    path.display().to_string()
}

fn config_with_home(home: &TempDir) -> Config {
    Config::from_sources(
        Some("http://127.0.0.1:3005".into()),
        Some("https://app.vibe.engineering".into()),
        Some(home.path().as_os_str().to_owned()),
        Some("claude".into()),
    )
    .unwrap()
}

fn request() -> ProviderRunRequest {
    ProviderRunRequest {
        provider_session_id: "provider-1".into(),
        prompt: "hello".into(),
        working_dir: "/tmp/vibe-cli-provider".into(),
        resume: false,
        sandbox_mode: SandboxMode::Disabled,
        output_sender: None,
        started_sender: None,
    }
}

#[tokio::test]
async fn codex_backend_runs_mock_binary() {
    let home = TempDir::new().unwrap();
    let script = make_script(
        &home,
        "mock-codex.sh",
        "#!/usr/bin/env bash\nset -euo pipefail\necho codex:$*\n",
    );
    let mut config = config_with_home(&home);
    config.codex_bin = script;
    let result = CodexBackend::new(&config).run(request()).await.unwrap();
    assert!(result.output.contains("codex:exec"));
}

#[tokio::test]
async fn gemini_backend_runs_mock_binary() {
    let home = TempDir::new().unwrap();
    let script = make_script(
        &home,
        "mock-gemini.sh",
        "#!/usr/bin/env bash\nset -euo pipefail\necho gemini:$*\n",
    );
    let mut config = config_with_home(&home);
    config.gemini_bin = script;
    let result = GeminiBackend::new(&config).run(request()).await.unwrap();
    assert!(result.output.contains("gemini:hello"));
}

#[tokio::test]
async fn openclaw_backend_runs_mock_binary() {
    let home = TempDir::new().unwrap();
    let script = make_script(
        &home,
        "mock-openclaw.sh",
        "#!/usr/bin/env bash\nset -euo pipefail\necho openclaw:$*\n",
    );
    let mut config = config_with_home(&home);
    config.openclaw_bin = script;
    let result = OpenclawBackend::new(&config).run(request()).await.unwrap();
    assert!(result.output.contains("openclaw:hello"));
}

#[tokio::test]
async fn acp_backend_runs_mock_binary() {
    let home = TempDir::new().unwrap();
    let script = make_script(
        &home,
        "mock-acp.sh",
        "#!/usr/bin/env bash\nset -euo pipefail\necho acp:$*\n",
    );
    let mut config = config_with_home(&home);
    config.acp_bin = script;
    let result = AcpBackend::new(&config).run(request()).await.unwrap();
    assert!(result.output.contains("acp:hello"));
}

#[tokio::test]
async fn provider_registry_dispatches_to_configured_backends() {
    let home = TempDir::new().unwrap();
    let codex = make_script(
        &home,
        "mock-codex.sh",
        "#!/usr/bin/env bash\nset -euo pipefail\necho registry-codex:$*\n",
    );
    let mut config = config_with_home(&home);
    config.codex_bin = codex;
    let registry = ProviderRegistry::new(&config);
    let result = registry
        .run(vibe_cli::agent::core::ProviderKind::Codex, request())
        .await
        .unwrap();
    assert!(result.output.contains("registry-codex:exec"));
}
