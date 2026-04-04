use std::{
    process::{Command, Output},
    time::Duration,
};

use serde_json::{Value, json};
use tempfile::TempDir;
use vibe_agent::{
    api::ApiClient,
    config::Config as AgentConfig,
    credentials::write_credentials as write_agent_credentials,
    encryption::{
        EncryptionVariant, derive_content_key_pair, encode_base64, encrypt_json,
        wrap_data_encryption_key,
    },
};
use vibe_cli::{
    config::Config as CliConfig,
    credentials::{read_credentials, write_credentials_data_key},
};

mod fixtures;

use fixtures::{TestServer, run_cli_with_envs, stderr, stdout, write_executable};

fn run_cli(home: &TempDir, server_url: &str, claude_bin: &str, args: &[&str]) -> Output {
    run_cli_with_envs(home, server_url, &[("VIBE_CLAUDE_BIN", claude_bin)], args)
}

fn create_mock_claude(home: &TempDir) -> String {
    let script = home.path().join("mock-claude.sh");
    write_executable(
        &script,
        r#"#!/usr/bin/env bash
set -euo pipefail
mode="new"
sid=""
prompt=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    -p) shift ;;
    --session-id) sid="$2"; shift 2 ;;
    --resume|-r) mode="resume"; sid="$2"; shift 2 ;;
    --add-dir) shift 2 ;;
    *) prompt="$1"; shift ;;
  esac
done
printf "provider=%s session=%s prompt=%s" "$mode" "$sid" "$prompt"
"#,
    );
    script.display().to_string()
}

fn create_mock_codex(home: &TempDir) -> String {
    let script = home.path().join("mock-codex.sh");
    write_executable(
        &script,
        r#"#!/usr/bin/env bash
set -euo pipefail
mode="new"
sid=""
prompt=""
if [[ "${1:-}" == "exec" ]]; then
  shift
fi
if [[ "${1:-}" == "resume" ]]; then
  mode="resume"
  sid="${2:-}"
else
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --json) shift ;;
      *) prompt="$1"; shift ;;
    esac
  done
fi
printf "provider=%s session=%s prompt=%s" "$mode" "$sid" "$prompt"
"#,
    );
    script.display().to_string()
}

fn create_mock_generic_provider(home: &TempDir, filename: &str, prefix: &str) -> String {
    let script = home.path().join(filename);
    let body = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\nprovider=\"{prefix}\"\nmode=\"new\"\nsid=\"\"\nprompt=\"\"\nwhile [[ $# -gt 0 ]]; do\n  case \"$1\" in\n    --resume) mode=\"resume\"; sid=\"$2\"; shift 2 ;;\n    --session-id) sid=\"$2\"; shift 2 ;;\n    *) prompt=\"$1\"; shift ;;\n  esac\ndone\nprintf \"provider=%s mode=%s session=%s prompt=%s\" \"$provider\" \"$mode\" \"$sid\" \"$prompt\"\n",
        prefix = prefix,
    );
    write_executable(&script, &body);
    script.display().to_string()
}

fn create_multiline_claude(home: &TempDir) -> String {
    let script = home.path().join("multiline-claude.sh");
    write_executable(
        &script,
        r#"#!/usr/bin/env bash
set -euo pipefail
printf "first line\n"
sleep 0.1
printf "second line\n"
"#,
    );
    script.display().to_string()
}

fn create_structured_claude(home: &TempDir) -> String {
    let script = home.path().join("structured-claude.sh");
    write_executable(
        &script,
        r#"#!/usr/bin/env bash
set -euo pipefail
printf '{"type":"thinking","text":"planning"}\n'
printf '{"type":"tool-call-start","call":"call-1","name":"shell","title":"Run ls","description":"Run ls","args":{"command":"ls"}}\n'
printf '{"type":"tool-call-end","call":"call-1"}\n'
printf '{"type":"file","ref":"file-1","name":"demo.txt","size":4,"mimeType":"text/plain"}\n'
printf '{"type":"text","text":"done"}\n'
"#,
    );
    script.display().to_string()
}

fn create_failing_claude(home: &TempDir) -> String {
    let script = home.path().join("failing-claude.sh");
    write_executable(
        &script,
        r#"#!/usr/bin/env bash
set -euo pipefail
echo "provider failed" >&2
exit 7
"#,
    );
    script.display().to_string()
}

fn create_slow_claude(home: &TempDir) -> String {
    let script = home.path().join("slow-claude.sh");
    write_executable(
        &script,
        r#"#!/usr/bin/env bash
set -euo pipefail
echo "started"
sleep 10
echo "finished"
"#,
    );
    script.display().to_string()
}

#[tokio::test(flavor = "multi_thread")]
async fn claude_run_and_resume_round_trip_against_real_vibe_server() {
    let server = TestServer::start().await;
    let home = TempDir::new().unwrap();
    server.provision_credentials(&home);
    let claude_bin = create_mock_claude(&home);

    let create = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &[
            "claude",
            "run",
            "--tag",
            "wave5-demo",
            "--path",
            "/tmp/wave5-demo",
            "--prompt",
            "hello",
            "--json",
        ],
    );
    assert!(create.status.success(), "{}", stderr(&create));
    let created: Value = serde_json::from_str(&stdout(&create)).unwrap();
    let session_id = created["sessionId"].as_str().unwrap().to_string();
    let provider_session_id = created["providerSessionId"].as_str().unwrap().to_string();
    assert_eq!(created["provider"], "claude");

    let cli_config = CliConfig::from_sources(
        Some(server.server_url.clone()),
        Some("https://app.vibe.engineering".into()),
        Some(home.path().as_os_str().to_owned()),
        None,
    )
    .unwrap();
    let credentials = read_credentials(&cli_config).unwrap();
    let agent_config = AgentConfig {
        server_url: server.server_url.clone(),
        home_dir: cli_config.home_dir.clone(),
        credential_path: cli_config.credential_path.clone(),
    };
    let api = ApiClient::new(
        agent_config,
        credentials
            .as_legacy_agent_credentials()
            .expect("wave5 integration uses legacy test credentials"),
    );

    let sessions = api.list_sessions().await.unwrap();
    let session = sessions
        .iter()
        .find(|session| session.id == session_id)
        .unwrap();
    assert_eq!(session.metadata["provider"], "claude");

    let history = api
        .get_session_messages(&session.id, &session.encryption)
        .await
        .unwrap();
    assert!(
        history
            .iter()
            .any(|message| { message.content.get("role").and_then(Value::as_str) == Some("user") })
    );

    let resume = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &["resume", &session_id, "--prompt", "again", "--json"],
    );
    assert!(resume.status.success(), "{}", stderr(&resume));
    let resumed: Value = serde_json::from_str(&stdout(&resume)).unwrap();
    assert_eq!(resumed["resumed"], true);
    assert_eq!(resumed["providerSessionId"], provider_session_id);

    tokio::time::sleep(Duration::from_millis(200)).await;
    let sessions = api.list_sessions().await.unwrap();
    let session = sessions
        .iter()
        .find(|session| session.id == session_id)
        .unwrap();
    let history = api
        .get_session_messages(&session.id, &session.encryption)
        .await
        .unwrap();
    let texts = history
        .iter()
        .map(|message| message.content.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(texts.contains("provider=new"));
    assert!(texts.contains("provider=resume"));

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn daemon_start_registers_machine_against_real_vibe_server() {
    let server = TestServer::start().await;
    let home = TempDir::new().unwrap();
    server.provision_credentials(&home);
    let claude_bin = create_mock_claude(&home);

    let start = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &["daemon", "start", "--json"],
    );
    assert!(start.status.success(), "{}", stderr(&start));

    let cli_config = CliConfig::from_sources(
        Some(server.server_url.clone()),
        Some("https://app.vibe.engineering".into()),
        Some(home.path().as_os_str().to_owned()),
        None,
    )
    .unwrap();
    let credentials = read_credentials(&cli_config).unwrap();
    let api = ApiClient::new(
        AgentConfig {
            server_url: server.server_url.clone(),
            home_dir: cli_config.home_dir.clone(),
            credential_path: cli_config.credential_path.clone(),
        },
        credentials
            .as_legacy_agent_credentials()
            .expect("daemon integration uses legacy test credentials"),
    );

    let mut running_machine = None;
    for _ in 0..20 {
        let machines = api.list_machines().await.unwrap();
        if let Some(machine) = machines.into_iter().find(|machine| {
            machine
                .daemon_state
                .as_ref()
                .and_then(|state| state.get("status"))
                .and_then(Value::as_str)
                == Some("running")
        }) {
            running_machine = Some(machine);
            break;
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    let running_machine = running_machine.expect("daemon never registered a running machine state");
    assert!(running_machine.metadata.get("happyCliVersion").is_some());
    assert!(running_machine.metadata.get("happyHomeDir").is_some());
    assert!(running_machine.metadata.get("vibeCliVersion").is_none());
    assert!(
        running_machine.metadata["cliAvailability"]
            .get("acp")
            .is_some()
    );
    assert_eq!(
        running_machine.metadata["resumeSupport"]["rpcAvailable"],
        Value::Bool(false)
    );

    let first_started_at = running_machine
        .daemon_state
        .as_ref()
        .and_then(|state| state.get("startedAt"))
        .and_then(Value::as_u64)
        .expect("running daemon state should include startedAt");
    tokio::time::sleep(Duration::from_millis(1_200)).await;
    let refreshed_machine = api
        .list_machines()
        .await
        .unwrap()
        .into_iter()
        .find(|machine| machine.id == running_machine.id)
        .expect("running machine should still exist");
    let refreshed_started_at = refreshed_machine
        .daemon_state
        .as_ref()
        .and_then(|state| state.get("startedAt"))
        .and_then(Value::as_u64)
        .expect("refreshed daemon state should include startedAt");
    assert_eq!(refreshed_started_at, first_started_at);

    let stop = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &["daemon", "stop", "--json"],
    );
    assert!(stop.status.success(), "{}", stderr(&stop));

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn daemon_metadata_reports_resume_support_when_local_agent_credentials_exist() {
    let server = TestServer::start().await;
    let home = TempDir::new().unwrap();
    server.provision_credentials(&home);
    let claude_bin = create_mock_claude(&home);

    let cli_config = CliConfig::from_sources(
        Some(server.server_url.clone()),
        Some("https://app.vibe.engineering".into()),
        Some(home.path().as_os_str().to_owned()),
        None,
    )
    .unwrap();
    write_agent_credentials(
        &AgentConfig {
            server_url: server.server_url.clone(),
            home_dir: cli_config.home_dir.clone(),
            credential_path: cli_config.home_dir.join("agent.key"),
        },
        "agent-token",
        [9u8; 32],
    )
    .unwrap();

    let start = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &["daemon", "start", "--json"],
    );
    assert!(start.status.success(), "{}", stderr(&start));

    let credentials = read_credentials(&cli_config).unwrap();
    let api = ApiClient::new(
        AgentConfig {
            server_url: server.server_url.clone(),
            home_dir: cli_config.home_dir.clone(),
            credential_path: cli_config.credential_path.clone(),
        },
        credentials
            .as_legacy_agent_credentials()
            .expect("daemon integration uses legacy test credentials"),
    );

    let mut running_machine = None;
    for _ in 0..20 {
        let machines = api.list_machines().await.unwrap();
        if let Some(machine) = machines.into_iter().find(|machine| {
            machine
                .daemon_state
                .as_ref()
                .and_then(|state| state.get("status"))
                .and_then(Value::as_str)
                == Some("running")
        }) {
            running_machine = Some(machine);
            break;
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    let running_machine = running_machine.expect("daemon never registered a running machine state");
    assert_eq!(
        running_machine.metadata["resumeSupport"]["rpcAvailable"],
        Value::Bool(true)
    );
    assert_eq!(
        running_machine.metadata["resumeSupport"]["happyAgentAuthenticated"],
        Value::Bool(true)
    );
    assert_eq!(
        running_machine.metadata["resumeSupport"]["requiresHappyAgentAuth"],
        Value::Bool(true)
    );

    let stop = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &["daemon", "stop", "--json"],
    );
    assert!(stop.status.success(), "{}", stderr(&stop));

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn daemon_refreshes_machine_metadata_when_resume_support_changes() {
    let server = TestServer::start().await;
    let home = TempDir::new().unwrap();
    server.provision_credentials(&home);
    let claude_bin = create_mock_claude(&home);

    let cli_config = CliConfig::from_sources(
        Some(server.server_url.clone()),
        Some("https://app.vibe.engineering".into()),
        Some(home.path().as_os_str().to_owned()),
        None,
    )
    .unwrap();

    let start = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &["daemon", "start", "--json"],
    );
    assert!(start.status.success(), "{}", stderr(&start));

    let credentials = read_credentials(&cli_config).unwrap();
    let api = ApiClient::new(
        AgentConfig {
            server_url: server.server_url.clone(),
            home_dir: cli_config.home_dir.clone(),
            credential_path: cli_config.credential_path.clone(),
        },
        credentials
            .as_legacy_agent_credentials()
            .expect("daemon integration uses legacy test credentials"),
    );

    let mut running_machine = None;
    for _ in 0..20 {
        let machines = api.list_machines().await.unwrap();
        if let Some(machine) = machines.into_iter().find(|machine| {
            machine
                .daemon_state
                .as_ref()
                .and_then(|state| state.get("status"))
                .and_then(Value::as_str)
                == Some("running")
        }) {
            running_machine = Some(machine);
            break;
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    let running_machine = running_machine.expect("daemon never registered a running machine state");
    assert_eq!(
        running_machine.metadata["resumeSupport"]["rpcAvailable"],
        Value::Bool(false)
    );

    write_agent_credentials(
        &AgentConfig {
            server_url: server.server_url.clone(),
            home_dir: cli_config.home_dir.clone(),
            credential_path: cli_config.home_dir.join("agent.key"),
        },
        "agent-token",
        [8u8; 32],
    )
    .unwrap();

    let mut refreshed_machine = None;
    for _ in 0..20 {
        let machines = api.list_machines().await.unwrap();
        if let Some(machine) = machines.into_iter().find(|machine| {
            machine.id == running_machine.id
                && machine.metadata["resumeSupport"]["rpcAvailable"] == Value::Bool(true)
        }) {
            refreshed_machine = Some(machine);
            break;
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    let refreshed_machine =
        refreshed_machine.expect("daemon never refreshed machine metadata after auth changed");
    assert_eq!(
        refreshed_machine.metadata["resumeSupport"]["happyAgentAuthenticated"],
        Value::Bool(true)
    );

    let stop = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &["daemon", "stop", "--json"],
    );
    assert!(stop.status.success(), "{}", stderr(&stop));

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn daemon_lists_and_stops_registered_sessions() {
    let server = TestServer::start().await;
    let home = TempDir::new().unwrap();
    server.provision_credentials(&home);
    let claude_bin = create_mock_claude(&home);

    let start = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &["daemon", "start", "--json"],
    );
    assert!(start.status.success(), "{}", stderr(&start));

    let create = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &[
            "claude",
            "run",
            "--tag",
            "wave5-daemon-list",
            "--path",
            "/tmp/wave5-daemon-list",
            "--prompt",
            "hello",
            "--json",
        ],
    );
    assert!(create.status.success(), "{}", stderr(&create));
    let created: Value = serde_json::from_str(&stdout(&create)).unwrap();
    let session_id = created["sessionId"].as_str().unwrap().to_string();

    let list = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &["daemon", "list", "--json"],
    );
    assert!(list.status.success(), "{}", stderr(&list));
    let listed: Value = serde_json::from_str(&stdout(&list)).unwrap();
    assert!(
        listed
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["session_id"] == session_id)
    );

    let stop_session = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &["daemon", "stop-session", &session_id, "--json"],
    );
    assert!(stop_session.status.success(), "{}", stderr(&stop_session));
    let stopped: Value = serde_json::from_str(&stdout(&stop_session)).unwrap();
    assert_eq!(stopped["sessionId"], session_id);
    assert_eq!(stopped["stopped"], true);

    let stop = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &["daemon", "stop", "--json"],
    );
    assert!(stop.status.success(), "{}", stderr(&stop));

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn codex_run_and_resume_round_trip_against_real_vibe_server() {
    let server = TestServer::start().await;
    let home = TempDir::new().unwrap();
    server.provision_credentials(&home);
    let codex_bin = create_mock_codex(&home);

    let create = Command::new(env!("CARGO_BIN_EXE_vibe"))
        .args([
            "codex",
            "run",
            "--tag",
            "wave5-codex",
            "--path",
            "/tmp/wave5-codex",
            "--prompt",
            "hello",
            "--json",
        ])
        .env("VIBE_HOME_DIR", home.path())
        .env("VIBE_SERVER_URL", &server.server_url)
        .env("VIBE_CODEX_BIN", &codex_bin)
        .output()
        .unwrap();
    assert!(create.status.success(), "{}", stderr(&create));
    let created: Value = serde_json::from_str(&stdout(&create)).unwrap();
    let session_id = created["sessionId"].as_str().unwrap().to_string();
    let provider_session_id = created["providerSessionId"].as_str().unwrap().to_string();

    let resume = Command::new(env!("CARGO_BIN_EXE_vibe"))
        .args(["resume", &session_id, "--prompt", "again", "--json"])
        .env("VIBE_HOME_DIR", home.path())
        .env("VIBE_SERVER_URL", &server.server_url)
        .env("VIBE_CODEX_BIN", &codex_bin)
        .output()
        .unwrap();
    assert!(resume.status.success(), "{}", stderr(&resume));
    let resumed: Value = serde_json::from_str(&stdout(&resume)).unwrap();
    assert_eq!(resumed["providerSessionId"], provider_session_id);
    assert_eq!(resumed["provider"], "codex");

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn gemini_run_and_resume_round_trip_against_real_vibe_server() {
    let server = TestServer::start().await;
    let home = TempDir::new().unwrap();
    server.provision_credentials(&home);
    let gemini_bin = create_mock_generic_provider(&home, "mock-gemini.sh", "gemini");

    let create = run_cli_with_envs(
        &home,
        &server.server_url,
        &[("VIBE_GEMINI_BIN", &gemini_bin)],
        &[
            "gemini",
            "run",
            "--tag",
            "wave5-gemini",
            "--path",
            "/tmp/wave5-gemini",
            "--prompt",
            "hello",
            "--json",
        ],
    );
    assert!(create.status.success(), "{}", stderr(&create));
    let created: Value = serde_json::from_str(&stdout(&create)).unwrap();
    let session_id = created["sessionId"].as_str().unwrap().to_string();
    let provider_session_id = created["providerSessionId"].as_str().unwrap().to_string();
    assert_eq!(created["provider"], "gemini");

    let resume = run_cli_with_envs(
        &home,
        &server.server_url,
        &[("VIBE_GEMINI_BIN", &gemini_bin)],
        &["resume", &session_id, "--prompt", "again", "--json"],
    );
    assert!(resume.status.success(), "{}", stderr(&resume));
    let resumed: Value = serde_json::from_str(&stdout(&resume)).unwrap();
    assert_eq!(resumed["providerSessionId"], provider_session_id);
    assert_eq!(resumed["provider"], "gemini");

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn openclaw_run_and_resume_round_trip_against_real_vibe_server() {
    let server = TestServer::start().await;
    let home = TempDir::new().unwrap();
    server.provision_credentials(&home);
    let openclaw_bin = create_mock_generic_provider(&home, "mock-openclaw.sh", "openclaw");

    let create = run_cli_with_envs(
        &home,
        &server.server_url,
        &[("VIBE_OPENCLAW_BIN", &openclaw_bin)],
        &[
            "openclaw",
            "run",
            "--tag",
            "wave5-openclaw",
            "--path",
            "/tmp/wave5-openclaw",
            "--prompt",
            "hello",
            "--json",
        ],
    );
    assert!(create.status.success(), "{}", stderr(&create));
    let created: Value = serde_json::from_str(&stdout(&create)).unwrap();
    let session_id = created["sessionId"].as_str().unwrap().to_string();
    let provider_session_id = created["providerSessionId"].as_str().unwrap().to_string();
    assert_eq!(created["provider"], "openclaw");

    let resume = run_cli_with_envs(
        &home,
        &server.server_url,
        &[("VIBE_OPENCLAW_BIN", &openclaw_bin)],
        &["resume", &session_id, "--prompt", "again", "--json"],
    );
    assert!(resume.status.success(), "{}", stderr(&resume));
    let resumed: Value = serde_json::from_str(&stdout(&resume)).unwrap();
    assert_eq!(resumed["providerSessionId"], provider_session_id);
    assert_eq!(resumed["provider"], "openclaw");

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn acp_run_and_resume_round_trip_against_real_vibe_server() {
    let server = TestServer::start().await;
    let home = TempDir::new().unwrap();
    server.provision_credentials(&home);
    let acp_bin = create_mock_generic_provider(&home, "mock-acp.sh", "acp");

    let create = run_cli_with_envs(
        &home,
        &server.server_url,
        &[("VIBE_ACP_BIN", &acp_bin)],
        &[
            "acp",
            "run",
            "--tag",
            "wave5-acp",
            "--path",
            "/tmp/wave5-acp",
            "--prompt",
            "hello",
            "--json",
        ],
    );
    assert!(create.status.success(), "{}", stderr(&create));
    let created: Value = serde_json::from_str(&stdout(&create)).unwrap();
    let session_id = created["sessionId"].as_str().unwrap().to_string();
    let provider_session_id = created["providerSessionId"].as_str().unwrap().to_string();
    assert_eq!(created["provider"], "acp");

    let resume = run_cli_with_envs(
        &home,
        &server.server_url,
        &[("VIBE_ACP_BIN", &acp_bin)],
        &["resume", &session_id, "--prompt", "again", "--json"],
    );
    assert!(resume.status.success(), "{}", stderr(&resume));
    let resumed: Value = serde_json::from_str(&stdout(&resume)).unwrap();
    assert_eq!(resumed["providerSessionId"], provider_session_id);
    assert_eq!(resumed["provider"], "acp");

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn data_key_cli_can_decrypt_session_via_local_agent_key_fallback() {
    let server = TestServer::start().await;
    let home = TempDir::new().unwrap();

    let cli_config = CliConfig::from_sources(
        Some(server.server_url.clone()),
        Some("https://app.vibe.engineering".into()),
        Some(home.path().as_os_str().to_owned()),
        None,
    )
    .unwrap();

    let account = server
        .ctx
        .db()
        .upsert_account_by_public_key("wave5-data-key-user");
    let token = server.ctx.auth().create_token(&account.id, None);

    let cli_public_key = [9u8; 32];
    let cli_machine_key = [10u8; 32];
    write_credentials_data_key(&cli_config, token, cli_public_key, cli_machine_key).unwrap();

    let agent_secret = [7u8; 32];
    write_agent_credentials(
        &AgentConfig {
            server_url: server.server_url.clone(),
            home_dir: cli_config.home_dir.clone(),
            credential_path: cli_config.home_dir.join("agent.key"),
        },
        "agent-token",
        agent_secret,
    )
    .unwrap();
    let agent_content_keys = derive_content_key_pair(&agent_secret);

    let session_key = [11u8; 32];
    let wrapped = wrap_data_encryption_key(&session_key, &agent_content_keys.public_key);
    let encrypted_metadata = encode_base64(
        &encrypt_json(
            &session_key,
            EncryptionVariant::DataKey,
            &json!({
                "tag": "remote-data-key",
                "path": "/tmp/remote-data-key",
                "host": "other-machine",
            }),
        )
        .unwrap(),
    );
    let (session, _) = server.ctx.db().create_or_load_session(
        &account.id,
        "remote-data-key",
        &encrypted_metadata,
        Some(wrapped),
    );

    let status = Command::new(env!("CARGO_BIN_EXE_vibe"))
        .args(["sessions", "status", &session.id, "--json"])
        .env("VIBE_HOME_DIR", home.path())
        .env("VIBE_SERVER_URL", &server.server_url)
        .output()
        .unwrap();
    assert!(status.status.success(), "{}", stderr(&status));
    let parsed: Value = serde_json::from_str(&stdout(&status)).unwrap();
    assert_eq!(parsed["id"], session.id);
    assert_eq!(parsed["metadata"]["tag"], "remote-data-key");
    assert_eq!(parsed["metadata"]["path"], "/tmp/remote-data-key");

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn provider_run_refreshes_machine_metadata_when_resume_support_changes() {
    let server = TestServer::start().await;
    let home = TempDir::new().unwrap();
    server.provision_credentials(&home);
    let claude_bin = create_mock_claude(&home);

    let first_run = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &[
            "claude",
            "run",
            "--tag",
            "wave5-metadata-refresh-a",
            "--path",
            "/tmp/wave5-metadata-refresh-a",
            "--prompt",
            "hello",
            "--json",
        ],
    );
    assert!(first_run.status.success(), "{}", stderr(&first_run));

    let cli_config = CliConfig::from_sources(
        Some(server.server_url.clone()),
        Some("https://app.vibe.engineering".into()),
        Some(home.path().as_os_str().to_owned()),
        None,
    )
    .unwrap();
    let credentials = read_credentials(&cli_config).unwrap();
    let api = ApiClient::new(
        AgentConfig {
            server_url: server.server_url.clone(),
            home_dir: cli_config.home_dir.clone(),
            credential_path: cli_config.credential_path.clone(),
        },
        credentials
            .as_legacy_agent_credentials()
            .expect("wave5 integration uses legacy test credentials"),
    );
    let machine = api
        .list_machines()
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("provider run should register a machine");
    assert_eq!(
        machine.metadata["resumeSupport"]["rpcAvailable"],
        Value::Bool(false)
    );

    write_agent_credentials(
        &AgentConfig {
            server_url: server.server_url.clone(),
            home_dir: cli_config.home_dir.clone(),
            credential_path: cli_config.home_dir.join("agent.key"),
        },
        "agent-token",
        [6u8; 32],
    )
    .unwrap();

    let second_run = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &[
            "claude",
            "run",
            "--tag",
            "wave5-metadata-refresh-b",
            "--path",
            "/tmp/wave5-metadata-refresh-b",
            "--prompt",
            "again",
            "--json",
        ],
    );
    assert!(second_run.status.success(), "{}", stderr(&second_run));

    let refreshed_machine = api
        .list_machines()
        .await
        .unwrap()
        .into_iter()
        .find(|candidate| candidate.id == machine.id)
        .expect("machine should still exist");
    assert_eq!(
        refreshed_machine.metadata["resumeSupport"]["rpcAvailable"],
        Value::Bool(true)
    );
    assert_eq!(
        refreshed_machine.metadata["resumeSupport"]["happyAgentAuthenticated"],
        Value::Bool(true)
    );

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn connect_status_json_does_not_expose_vendor_tokens() {
    let server = TestServer::start().await;
    let home = TempDir::new().unwrap();
    server.provision_credentials(&home);
    let claude_bin = create_mock_claude(&home);

    let register = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &[
            "auth",
            "connect",
            "register",
            "anthropic",
            "--token",
            "super-secret-token",
            "--json",
        ],
    );
    assert!(register.status.success(), "{}", stderr(&register));

    let status = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &["auth", "connect", "status", "--json"],
    );
    assert!(status.status.success(), "{}", stderr(&status));
    let output = stdout(&status);
    assert!(output.contains("\"vendor\": \"anthropic\""));
    assert!(output.contains("\"connected\": true"));
    assert!(!output.contains("super-secret-token"));

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn provider_failure_clears_agent_busy_state() {
    let server = TestServer::start().await;
    let home = TempDir::new().unwrap();
    server.provision_credentials(&home);
    let claude_bin = create_failing_claude(&home);

    let create = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &[
            "claude",
            "run",
            "--tag",
            "wave5-failure",
            "--path",
            "/tmp/wave5-failure",
            "--prompt",
            "boom",
            "--json",
        ],
    );
    assert!(!create.status.success());
    assert!(stderr(&create).contains("provider failed"));

    let cli_config = CliConfig::from_sources(
        Some(server.server_url.clone()),
        Some("https://app.vibe.engineering".into()),
        Some(home.path().as_os_str().to_owned()),
        None,
    )
    .unwrap();
    let credentials = read_credentials(&cli_config).unwrap();
    let agent_config = AgentConfig {
        server_url: server.server_url.clone(),
        home_dir: cli_config.home_dir.clone(),
        credential_path: cli_config.credential_path.clone(),
    };
    let api = ApiClient::new(
        agent_config,
        credentials
            .as_legacy_agent_credentials()
            .expect("wave5 integration uses legacy test credentials"),
    );

    let session = api
        .list_sessions()
        .await
        .unwrap()
        .into_iter()
        .find(|session| session.metadata["tag"] == "wave5-failure")
        .unwrap();
    assert_eq!(
        session
            .agent_state
            .as_ref()
            .and_then(|state| state.get("requests"))
            .cloned()
            .unwrap_or(Value::Null),
        json!({})
    );

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn workspace_sandbox_fails_clearly_for_unsupported_providers() {
    let server = TestServer::start().await;
    let home = TempDir::new().unwrap();
    server.provision_credentials(&home);
    let codex_bin = create_mock_codex(&home);

    let configure = run_cli_with_envs(
        &home,
        &server.server_url,
        &[("VIBE_CODEX_BIN", &codex_bin)],
        &["sandbox", "configure", "--mode", "workspace", "--json"],
    );
    assert!(configure.status.success(), "{}", stderr(&configure));

    let create = run_cli_with_envs(
        &home,
        &server.server_url,
        &[("VIBE_CODEX_BIN", &codex_bin)],
        &[
            "codex",
            "run",
            "--tag",
            "wave5-codex-workspace",
            "--path",
            "/tmp/wave5-codex-workspace",
            "--prompt",
            "hello",
            "--json",
        ],
    );
    assert!(!create.status.success());
    assert!(stderr(&create).contains("workspace sandbox is not supported"));

    let cli_config = CliConfig::from_sources(
        Some(server.server_url.clone()),
        Some("https://app.vibe.engineering".into()),
        Some(home.path().as_os_str().to_owned()),
        None,
    )
    .unwrap();
    let credentials = read_credentials(&cli_config).unwrap();
    let api = ApiClient::new(
        AgentConfig {
            server_url: server.server_url.clone(),
            home_dir: cli_config.home_dir.clone(),
            credential_path: cli_config.credential_path.clone(),
        },
        credentials
            .as_legacy_agent_credentials()
            .expect("wave5 integration uses legacy test credentials"),
    );
    let sessions = api.list_sessions().await.unwrap();
    assert!(
        sessions.is_empty(),
        "unsupported sandbox should fail before session creation"
    );

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn daemon_stop_session_terminates_running_provider_process() {
    let server = TestServer::start().await;
    let home = TempDir::new().unwrap();
    server.provision_credentials(&home);
    let claude_bin = create_slow_claude(&home);

    let start = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &["daemon", "start", "--json"],
    );
    assert!(start.status.success(), "{}", stderr(&start));

    let mut child = Command::new(env!("CARGO_BIN_EXE_vibe"))
        .args([
            "claude",
            "run",
            "--tag",
            "wave5-daemon-stop-live",
            "--path",
            "/tmp/wave5-daemon-stop-live",
            "--prompt",
            "hello",
            "--json",
        ])
        .env("VIBE_HOME_DIR", home.path())
        .env("VIBE_SERVER_URL", &server.server_url)
        .env("VIBE_CLAUDE_BIN", &claude_bin)
        .spawn()
        .unwrap();

    let mut session_id = None;
    for _ in 0..30 {
        let list = run_cli(
            &home,
            &server.server_url,
            &claude_bin,
            &["daemon", "list", "--json"],
        );
        assert!(list.status.success(), "{}", stderr(&list));
        let listed: Value = serde_json::from_str(&stdout(&list)).unwrap();
        session_id = listed
            .as_array()
            .and_then(|items| items.first())
            .and_then(|item| item["session_id"].as_str())
            .map(ToOwned::to_owned);
        if session_id.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    let session_id = session_id.expect("daemon should register the running session");

    let stop_session = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &["daemon", "stop-session", &session_id, "--json"],
    );
    assert!(stop_session.status.success(), "{}", stderr(&stop_session));
    let stopped: Value = serde_json::from_str(&stdout(&stop_session)).unwrap();
    assert_eq!(stopped["stopped"], true);

    let mut exit_status = None;
    for _ in 0..50 {
        if let Some(status) = child.try_wait().unwrap() {
            exit_status = Some(status);
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    let status = exit_status.expect("running CLI should exit after daemon stop");
    assert!(
        !status.success(),
        "running CLI should be interrupted by daemon stop"
    );

    let stop = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &["daemon", "stop", "--json"],
    );
    assert!(stop.status.success(), "{}", stderr(&stop));

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn multiline_provider_output_streams_multiple_text_events() {
    let server = TestServer::start().await;
    let home = TempDir::new().unwrap();
    server.provision_credentials(&home);
    let claude_bin = create_multiline_claude(&home);

    let create = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &[
            "claude",
            "run",
            "--tag",
            "wave5-stream",
            "--path",
            "/tmp/wave5-stream",
            "--prompt",
            "hello",
            "--json",
        ],
    );
    assert!(create.status.success(), "{}", stderr(&create));
    let created: Value = serde_json::from_str(&stdout(&create)).unwrap();
    let session_id = created["sessionId"].as_str().unwrap().to_string();

    let cli_config = CliConfig::from_sources(
        Some(server.server_url.clone()),
        Some("https://app.vibe.engineering".into()),
        Some(home.path().as_os_str().to_owned()),
        None,
    )
    .unwrap();
    let credentials = read_credentials(&cli_config).unwrap();
    let agent_config = AgentConfig {
        server_url: server.server_url.clone(),
        home_dir: cli_config.home_dir.clone(),
        credential_path: cli_config.credential_path.clone(),
    };
    let api = ApiClient::new(
        agent_config,
        credentials
            .as_legacy_agent_credentials()
            .expect("wave5 integration uses legacy test credentials"),
    );
    let sessions = api.list_sessions().await.unwrap();
    let session = sessions
        .iter()
        .find(|session| session.id == session_id)
        .unwrap();
    let history = api
        .get_session_messages(&session.id, &session.encryption)
        .await
        .unwrap();
    let text_events = history
        .iter()
        .filter(|message| {
            message.content["role"] == "session" && message.content["content"]["ev"]["t"] == "text"
        })
        .count();
    assert!(text_events >= 2, "expected multiple streamed text events");

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn structured_provider_events_round_trip_to_session_protocol() {
    let server = TestServer::start().await;
    let home = TempDir::new().unwrap();
    server.provision_credentials(&home);
    let claude_bin = create_structured_claude(&home);

    let create = run_cli(
        &home,
        &server.server_url,
        &claude_bin,
        &[
            "claude",
            "run",
            "--tag",
            "wave5-structured",
            "--path",
            "/tmp/wave5-structured",
            "--prompt",
            "hello",
            "--json",
        ],
    );
    assert!(create.status.success(), "{}", stderr(&create));
    let created: Value = serde_json::from_str(&stdout(&create)).unwrap();
    let session_id = created["sessionId"].as_str().unwrap().to_string();

    let cli_config = CliConfig::from_sources(
        Some(server.server_url.clone()),
        Some("https://app.vibe.engineering".into()),
        Some(home.path().as_os_str().to_owned()),
        None,
    )
    .unwrap();
    let credentials = read_credentials(&cli_config).unwrap();
    let api = ApiClient::new(
        AgentConfig {
            server_url: server.server_url.clone(),
            home_dir: cli_config.home_dir.clone(),
            credential_path: cli_config.credential_path.clone(),
        },
        credentials
            .as_legacy_agent_credentials()
            .expect("wave5 integration uses legacy test credentials"),
    );
    let sessions = api.list_sessions().await.unwrap();
    let session = sessions
        .iter()
        .find(|session| session.id == session_id)
        .unwrap();
    let history = api
        .get_session_messages(&session.id, &session.encryption)
        .await
        .unwrap();

    assert!(history.iter().any(|message| {
        message.content["role"] == "session"
            && message.content["content"]["ev"]["t"] == "text"
            && message.content["content"]["ev"]["thinking"] == Value::Bool(true)
    }));
    assert!(history.iter().any(|message| {
        message.content["role"] == "session"
            && message.content["content"]["ev"]["t"] == "tool-call-start"
    }));
    assert!(history.iter().any(|message| {
        message.content["role"] == "session"
            && message.content["content"]["ev"]["t"] == "tool-call-end"
    }));
    assert!(history.iter().any(|message| {
        message.content["role"] == "session"
            && message.content["content"]["ev"]["t"] == "file"
            && message.content["content"]["ev"]["name"] == "demo.txt"
    }));
    assert!(history.iter().any(|message| {
        message.content["role"] == "session"
            && message.content["content"]["ev"]["t"] == "text"
            && message.content["content"]["ev"]["text"] == "done"
    }));

    server.shutdown().await;
}
