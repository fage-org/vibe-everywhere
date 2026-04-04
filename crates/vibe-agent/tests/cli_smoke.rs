use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;
use vibe_agent::{config::Config, credentials::write_credentials, encryption::encode_base64};

fn run_cli(home: &TempDir, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vibe-agent"))
        .args(args)
        .env("VIBE_HOME_DIR", home.path())
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

#[test]
fn root_help_and_version_are_exposed() {
    let home = TempDir::new().unwrap();

    let help = run_cli(&home, &["--help"]);
    assert!(help.status.success());
    let help = stdout(&help);
    assert!(help.contains("CLI client for controlling Vibe agents remotely"));
    assert!(help.contains("Manage authentication"));
    assert!(help.contains("List all sessions"));
    assert!(help.contains("Get live session state"));

    let version = run_cli(&home, &["--version"]);
    assert!(version.status.success());
    assert_eq!(
        stdout(&version).trim(),
        format!("vibe-agent {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn list_help_describes_flags() {
    let home = TempDir::new().unwrap();

    let output = run_cli(&home, &["list", "--help"]);
    assert!(output.status.success());
    let output = stdout(&output);

    assert!(output.contains("List all sessions"));
    assert!(output.contains("Show only active sessions"));
    assert!(output.contains("Output as JSON"));
}

#[test]
fn unauthenticated_commands_show_reauthentication_error() {
    let home = TempDir::new().unwrap();

    let output = run_cli(&home, &["list"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("vibe-agent auth login"));
}

#[test]
fn auth_status_json_uses_persisted_content_public_key() {
    let home = TempDir::new().unwrap();
    let config = Config::from_sources(None, Some(home.path().as_os_str().to_owned())).unwrap();
    let secret = [9u8; 32];
    write_credentials(&config, "token-1", secret).unwrap();
    let credentials = vibe_agent::credentials::read_credentials(&config).unwrap();

    let output = run_cli(&home, &["auth", "status", "--json"]);
    assert!(output.status.success());

    let parsed: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(parsed["status"], "authenticated");
    assert_eq!(
        parsed["publicKey"],
        Value::String(encode_base64(&credentials.content_key_pair.public_key))
    );
}
