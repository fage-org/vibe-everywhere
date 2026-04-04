use serde_json::Value;
use tempfile::TempDir;

mod fixtures;

use fixtures::{run_cli, stderr, stdout};

#[test]
fn root_help_lists_wave5_command_families() {
    let home = TempDir::new().unwrap();
    let output = run_cli(&home, &["--help"]);
    assert!(output.status.success());
    let help = stdout(&output);
    assert!(help.contains("Vibe local runtime CLI"));
    assert!(help.contains("auth"));
    assert!(help.contains("sandbox"));
    assert!(help.contains("daemon"));
    assert!(help.contains("claude"));
    assert!(help.contains("resume"));
}

#[test]
fn unauthenticated_session_commands_fail_with_auth_hint() {
    let home = TempDir::new().unwrap();
    let output = run_cli(&home, &["sessions", "list"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("vibe auth login"));
}

#[test]
fn sandbox_commands_round_trip_via_cli() {
    let home = TempDir::new().unwrap();

    let configure = run_cli(
        &home,
        &["sandbox", "configure", "--mode", "workspace", "--json"],
    );
    assert!(configure.status.success(), "{}", stderr(&configure));
    let configured: Value = serde_json::from_str(&stdout(&configure)).unwrap();
    assert_eq!(configured["configured"], true);
    assert_eq!(configured["mode"], "workspace");

    let status = run_cli(&home, &["sandbox", "status", "--json"]);
    assert!(status.status.success(), "{}", stderr(&status));
    let status_json: Value = serde_json::from_str(&stdout(&status)).unwrap();
    assert_eq!(status_json["mode"], "workspace");

    let disable = run_cli(&home, &["sandbox", "disable", "--json"]);
    assert!(disable.status.success(), "{}", stderr(&disable));
    let disabled: Value = serde_json::from_str(&stdout(&disable)).unwrap();
    assert_eq!(disabled["mode"], "disabled");
}

#[test]
fn daemon_start_status_and_stop_work() {
    let home = TempDir::new().unwrap();

    let install = run_cli(&home, &["daemon", "install", "--json"]);
    assert!(install.status.success(), "{}", stderr(&install));
    let installed: Value = serde_json::from_str(&stdout(&install)).unwrap();
    assert_eq!(installed["installed"], true);

    let status = run_cli(&home, &["daemon", "status", "--json"]);
    assert!(status.status.success(), "{}", stderr(&status));
    let status_json: Value = serde_json::from_str(&stdout(&status)).unwrap();
    assert_eq!(status_json["running"], false);
    assert_eq!(status_json["installed"], true);

    let stop = run_cli(&home, &["daemon", "stop", "--json"]);
    assert!(stop.status.success(), "{}", stderr(&stop));
    let stopped: Value = serde_json::from_str(&stdout(&stop)).unwrap();
    assert_eq!(stopped["stopped"], false);

    let uninstall = run_cli(&home, &["daemon", "uninstall", "--json"]);
    assert!(uninstall.status.success(), "{}", stderr(&uninstall));
    let removed: Value = serde_json::from_str(&stdout(&uninstall)).unwrap();
    assert_eq!(removed["uninstalled"], true);
}
