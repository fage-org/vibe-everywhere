//! Tests for the Claude Code driver module.

use std::sync::Arc;

use super::*;
use crate::config::Config;

#[test]
fn test_stream_json_event_parse_partial() {
    let json = r#"{"type":"partial","content":"Hello"}"#;
    let event: StreamJsonEvent = serde_json::from_str(json).unwrap();
    match event {
        StreamJsonEvent::Partial { content } => {
            assert_eq!(content, "Hello");
        }
        _ => panic!("Expected Partial event"),
    }
}

#[test]
fn test_stream_json_event_parse_message() {
    let json = r#"{"type":"message","message":{"role":"assistant","content":[{"type":"text","text":"Hello world"}]}}"#;
    let event: StreamJsonEvent = serde_json::from_str(json).unwrap();
    match event {
        StreamJsonEvent::Message { message } => {
            assert_eq!(message.role, "assistant");
            assert_eq!(message.content.len(), 1);
            assert_eq!(message.content[0].text, Some("Hello world".to_string()));
        }
        _ => panic!("Expected Message event"),
    }
}

#[test]
fn test_stream_json_event_parse_tool_use() {
    let json = r#"{"type":"tool_use","tool_name":"bash","tool_input":{"command":"ls"}}"#;
    let event: StreamJsonEvent = serde_json::from_str(json).unwrap();
    match event {
        StreamJsonEvent::ToolUse {
            tool_name,
            tool_input,
        } => {
            assert_eq!(tool_name, "bash");
            assert_eq!(tool_input["command"], "ls");
        }
        _ => panic!("Expected ToolUse event"),
    }
}

#[test]
fn test_stream_json_event_parse_error() {
    let json = r#"{"type":"error","message":"Something went wrong"}"#;
    let event: StreamJsonEvent = serde_json::from_str(json).unwrap();
    match event {
        StreamJsonEvent::Error { message } => {
            assert_eq!(message, "Something went wrong");
        }
        _ => panic!("Expected Error event"),
    }
}

#[test]
fn interrupt_returns_error_without_child_process() {
    let config = Arc::new(Config {
        server_url: "https://test.com".to_string(),
        host_name: "test".to_string(),
        platform: "linux".to_string(),
        config_dir: std::path::PathBuf::from("/tmp"),
        log_format: "pretty".to_string(),
        log_level: "info".to_string(),
        heartbeat_interval_secs: 30,
        heartbeat_timeout_secs: 90,
        ack_timeout_secs: 30,
        permission_timeout_secs: 60,
        reconnect_backoff_min_ms: 1000,
        reconnect_backoff_max_ms: 30000,
        max_parallel_sessions: 4,
        file_read_text_limit_bytes: 262_144,
        file_tree_max_nodes: 20_000,
        claude_command: "claude".to_string(),
        default_model: "claude-sonnet-4-20250514".to_string(),
        permission_mode: "default".to_string(),
        mock_mode: false,
    });
    let (event_tx, _event_rx) = tokio::sync::broadcast::channel(8);
    let mut driver = ClaudeCodeDriver::new(config, event_tx);

    let error = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(driver.control(Uuid::nil(), SessionControlAction::Interrupt))
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("Interrupt is not supported safely"));
}
