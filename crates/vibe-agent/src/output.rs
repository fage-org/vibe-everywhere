use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use crate::api::{DecryptedMachine, DecryptedMessage, DecryptedSession};

pub fn format_session_table(sessions: &[DecryptedSession]) -> String {
    if sessions.is_empty() {
        return "## Sessions\n\n- Total: 0\n- Items: none".into();
    }

    let sections = sessions
        .iter()
        .enumerate()
        .map(|(index, session)| {
            let name = session_summary(&session.metadata)
                .or_else(|| object_string(&session.metadata, "tag"))
                .unwrap_or_else(|| "-".into());
            let path = object_string(&session.metadata, "path").unwrap_or_else(|| "-".into());
            let status = if session.active { "active" } else { "inactive" };
            [
                format!("### Session {}", index + 1),
                format!("- ID: {}", markdown_inline(&session.id)),
                format!("- Name: {}", normalize_list_value(&name)),
                format!("- Path: {}", normalize_list_value(&path)),
                format!("- Status: {status}"),
                format!(
                    "- Last Active: {}",
                    normalize_list_value(&format_last_active(session.active_at))
                ),
            ]
            .join("\n")
        })
        .collect::<Vec<_>>();

    format!(
        "## Sessions\n\n- Total: {}\n\n{}",
        sessions.len(),
        sections.join("\n\n")
    )
}

pub fn format_machine_table(machines: &[DecryptedMachine]) -> String {
    if machines.is_empty() {
        return "## Machines\n\n- Total: 0\n- Items: none".into();
    }

    let sections = machines
        .iter()
        .enumerate()
        .map(|(index, machine)| {
            let host = object_string(&machine.metadata, "host").unwrap_or_else(|| "-".into());
            let platform =
                object_string(&machine.metadata, "platform").unwrap_or_else(|| "-".into());
            let home = object_string(&machine.metadata, "homeDir").unwrap_or_else(|| "-".into());
            let status = if machine.active {
                object_string(
                    machine.daemon_state.as_ref().unwrap_or(&Value::Null),
                    "status",
                )
                .unwrap_or_else(|| "online".into())
            } else {
                "offline".into()
            };
            [
                format!("### Machine {}", index + 1),
                format!("- ID: {}", markdown_inline(&machine.id)),
                format!("- Host: {}", normalize_list_value(&host)),
                format!("- Platform: {}", normalize_list_value(&platform)),
                format!("- Status: {}", normalize_list_value(&status)),
                format!("- Home: {}", normalize_list_value(&home)),
                format!(
                    "- Last Active: {}",
                    normalize_list_value(&format_last_active(machine.active_at))
                ),
            ]
            .join("\n")
        })
        .collect::<Vec<_>>();

    format!(
        "## Machines\n\n- Total: {}\n\n{}",
        machines.len(),
        sections.join("\n\n")
    )
}

pub fn format_session_status(session: &DecryptedSession) -> String {
    let mut lines = vec![
        "## Session Status".to_string(),
        String::new(),
        format!("- Session ID: {}", markdown_inline(&session.id)),
    ];

    if let Some(tag) = object_string(&session.metadata, "tag") {
        lines.push(format!("- Tag: {tag}"));
    }
    if let Some(summary) = session_summary(&session.metadata) {
        lines.push(format!("- Summary: {summary}"));
    }
    if let Some(path) = object_string(&session.metadata, "path") {
        lines.push(format!("- Path: {path}"));
    }
    if let Some(host) = object_string(&session.metadata, "host") {
        lines.push(format!("- Host: {host}"));
    }
    if let Some(lifecycle_state) = object_string(&session.metadata, "lifecycleState") {
        lines.push(format!("- Lifecycle: {lifecycle_state}"));
    }
    lines.push(format!(
        "- Active: {}",
        if session.active { "yes" } else { "no" }
    ));
    lines.push(format!(
        "- Last Active: {}",
        format_last_active(session.active_at)
    ));

    if let Some(agent_state) = session.agent_state.as_ref().and_then(Value::as_object) {
        let requests = agent_state
            .get("requests")
            .and_then(Value::as_object)
            .map(|requests| requests.len())
            .unwrap_or(0);
        let busy = agent_state
            .get("controlledByUser")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            || requests > 0;
        lines.push(format!("- Agent: {}", if busy { "busy" } else { "idle" }));
        if requests > 0 {
            lines.push(format!("- Pending Requests: {requests}"));
        }
    } else {
        lines.push("- Agent: no state".into());
    }

    lines.join("\n")
}

pub fn format_message_history(messages: &[DecryptedMessage]) -> String {
    if messages.is_empty() {
        return "## Message History\n\n- Count: 0\n- Items: none".into();
    }

    let sections = messages
        .iter()
        .enumerate()
        .map(|(index, message)| {
            let role = object_string(&message.content, "role").unwrap_or_else(|| "unknown".into());
            let text = message_text(&message.content);
            [
                format!("### Message {}", index + 1),
                format!("- ID: {}", markdown_inline(&message.id)),
                format!("- Time: {}", format_iso_time(message.created_at)),
                format!("- Role: {role}"),
                "- Text:".into(),
                "```text".into(),
                normalize_code_block_text(&text),
                "```".into(),
            ]
            .join("\n")
        })
        .collect::<Vec<_>>();

    format!(
        "## Message History\n\n- Count: {}\n\n{}",
        messages.len(),
        sections.join("\n\n")
    )
}

pub fn format_json<T: Serialize>(data: &T) -> String {
    serde_json::to_string_pretty(data).unwrap_or_else(|_| "null".into())
}

fn object_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn session_summary(metadata: &Value) -> Option<String> {
    if let Some(summary) = metadata.get("summary").and_then(Value::as_str) {
        let trimmed = summary.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    metadata
        .get("summary")
        .and_then(Value::as_object)
        .and_then(|summary| summary.get("text"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn message_text(content: &Value) -> String {
    if let Some(text) = content
        .get("content")
        .and_then(Value::as_object)
        .and_then(|content| content.get("text"))
        .and_then(Value::as_str)
    {
        return text.to_string();
    }

    if let Some(text) = content.get("content").and_then(Value::as_str) {
        return text.to_string();
    }

    serde_json::to_string(content).unwrap_or_else(|_| "(invalid message)".into())
}

fn markdown_inline(value: &str) -> String {
    format!("`{}`", value.replace('`', "\\`"))
}

fn normalize_code_block_text(value: &str) -> String {
    let text = if value.trim().is_empty() {
        "(empty)"
    } else {
        value
    };
    text.replace("```", "``\\`")
}

fn normalize_list_value(value: &str) -> String {
    value.replace(['\n', '\r'], " ").trim().to_string()
}

fn format_time(ts: u64) -> String {
    if ts == 0 {
        return "-".into();
    }

    let now = Utc::now().timestamp_millis();
    let diff_ms = now.saturating_sub(ts as i64);
    let diff_min = diff_ms / 60_000;
    if diff_min < 1 {
        return "just now".into();
    }
    if diff_min < 60 {
        return format!("{diff_min}m ago");
    }
    let diff_hr = diff_min / 60;
    if diff_hr < 24 {
        return format!("{diff_hr}h ago");
    }
    let diff_day = diff_hr / 24;
    format!("{diff_day}d ago")
}

fn format_iso_time(ts: u64) -> String {
    if ts == 0 {
        return "-".into();
    }

    DateTime::<Utc>::from_timestamp_millis(ts as i64)
        .map(|value| value.to_rfc3339())
        .unwrap_or_else(|| "-".into())
}

fn format_last_active(ts: u64) -> String {
    let relative = format_time(ts);
    let absolute = format_iso_time(ts);
    if absolute == "-" {
        relative
    } else {
        format!("{relative} ({absolute})")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::api::{DecryptedMachine, DecryptedMessage, DecryptedSession, RecordEncryption};
    use crate::encryption::EncryptionVariant;

    use super::{
        format_json, format_machine_table, format_message_history, format_session_status,
        format_session_table,
    };

    fn sample_encryption() -> RecordEncryption {
        RecordEncryption {
            key: [0; 32],
            variant: EncryptionVariant::Legacy,
        }
    }

    fn sample_session() -> DecryptedSession {
        DecryptedSession {
            id: "session-1".into(),
            seq: 1,
            created_at: 0,
            updated_at: 0,
            active: true,
            active_at: 0,
            metadata: json!({"tag": "demo", "path": "/tmp/demo", "summary": "Demo"}),
            metadata_version: 1,
            agent_state: Some(json!({"controlledByUser": false, "requests": {}})),
            agent_state_version: 1,
            data_encryption_key: None,
            encryption: sample_encryption(),
        }
    }

    #[test]
    fn json_output_is_pretty() {
        let formatted = format_json(&sample_session());
        assert!(formatted.contains('\n'));
        assert!(!formatted.contains("dataEncryptionKey"));
        assert!(!formatted.contains("\"encryption\""));
        assert!(!formatted.contains("metadataVersion"));
        assert!(!formatted.contains("agentStateVersion"));
    }

    #[test]
    fn session_output_contains_expected_sections() {
        let output = format_session_table(&[sample_session()]);
        assert!(output.contains("## Sessions"));
        assert!(output.contains("Demo"));
    }

    #[test]
    fn machine_output_contains_expected_sections() {
        let machine = DecryptedMachine {
            id: "machine-1".into(),
            seq: 1,
            created_at: 0,
            updated_at: 0,
            active: true,
            active_at: 0,
            metadata: json!({"host": "demo-host", "platform": "linux", "homeDir": "/home/demo"}),
            metadata_version: 1,
            daemon_state: Some(json!({"status": "online"})),
            daemon_state_version: 1,
            data_encryption_key: None,
            encryption: sample_encryption(),
        };

        let output = format_machine_table(&[machine]);
        assert!(output.contains("## Machines"));
        assert!(output.contains("demo-host"));
    }

    #[test]
    fn session_status_mentions_agent_state() {
        let output = format_session_status(&sample_session());
        assert!(output.contains("## Session Status"));
        assert!(output.contains("- Agent: idle"));
    }

    #[test]
    fn history_output_renders_text_messages() {
        let output = format_message_history(&[DecryptedMessage {
            id: "msg-1".into(),
            seq: 1,
            content: json!({"role": "user", "content": {"type": "text", "text": "hello"}}),
            local_id: None,
            created_at: 0,
            updated_at: 0,
        }]);
        assert!(output.contains("## Message History"));
        assert!(output.contains("hello"));
    }

    #[test]
    fn session_table_output_is_stable() {
        assert_eq!(
            format_session_table(&[sample_session()]),
            "## Sessions\n\n- Total: 1\n\n### Session 1\n- ID: `session-1`\n- Name: Demo\n- Path: /tmp/demo\n- Status: active\n- Last Active: -"
        );
    }

    #[test]
    fn machine_table_output_is_stable() {
        let machine = DecryptedMachine {
            id: "machine-1".into(),
            seq: 1,
            created_at: 0,
            updated_at: 0,
            active: true,
            active_at: 0,
            metadata: json!({"host": "demo-host", "platform": "linux", "homeDir": "/home/demo"}),
            metadata_version: 1,
            daemon_state: Some(json!({"status": "online"})),
            daemon_state_version: 1,
            data_encryption_key: None,
            encryption: sample_encryption(),
        };

        assert_eq!(
            format_machine_table(&[machine]),
            "## Machines\n\n- Total: 1\n\n### Machine 1\n- ID: `machine-1`\n- Host: demo-host\n- Platform: linux\n- Status: online\n- Home: /home/demo\n- Last Active: -"
        );
    }

    #[test]
    fn session_status_output_is_stable() {
        assert_eq!(
            format_session_status(&sample_session()),
            "## Session Status\n\n- Session ID: `session-1`\n- Tag: demo\n- Summary: Demo\n- Path: /tmp/demo\n- Active: yes\n- Last Active: -\n- Agent: idle"
        );
    }

    #[test]
    fn history_output_is_stable() {
        let history = [DecryptedMessage {
            id: "msg-1".into(),
            seq: 1,
            content: json!({"role": "user", "content": {"type": "text", "text": "hello"}}),
            local_id: None,
            created_at: 0,
            updated_at: 0,
        }];

        assert_eq!(
            format_message_history(&history),
            "## Message History\n\n- Count: 1\n\n### Message 1\n- ID: `msg-1`\n- Time: -\n- Role: user\n- Text:\n```text\nhello\n```"
        );
    }
}
