use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedOutput {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NormalizedEvent {
    Text {
        text: String,
    },
    Thinking {
        text: String,
    },
    Service {
        text: String,
    },
    ToolCallStart {
        call: String,
        name: String,
        title: String,
        description: String,
        args: BTreeMap<String, Value>,
    },
    ToolCallEnd {
        call: String,
    },
    File {
        reference: String,
        name: String,
        size: u64,
        mime_type: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum ProviderEventLine {
    Text {
        text: String,
    },
    Thinking {
        text: String,
    },
    Service {
        text: String,
    },
    ToolCallStart {
        call: String,
        name: String,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        args: BTreeMap<String, Value>,
    },
    ToolCallEnd {
        call: String,
    },
    ToolCallResult {
        call: String,
    },
    File {
        #[serde(rename = "ref")]
        reference: String,
        name: String,
        size: u64,
        #[serde(rename = "mimeType", default)]
        mime_type: Option<String>,
    },
}

pub fn normalize_stdout(output: &str) -> NormalizedOutput {
    NormalizedOutput {
        text: output.trim().to_string(),
    }
}

pub fn normalize_output_line(line: &str) -> NormalizedEvent {
    let trimmed = line.trim();
    if let Ok(event) = serde_json::from_str::<ProviderEventLine>(trimmed) {
        return match event {
            ProviderEventLine::Text { text } => NormalizedEvent::Text { text },
            ProviderEventLine::Thinking { text } => NormalizedEvent::Thinking { text },
            ProviderEventLine::Service { text } => NormalizedEvent::Service { text },
            ProviderEventLine::ToolCallStart {
                call,
                name,
                title,
                description,
                args,
            } => {
                let title = title.unwrap_or_else(|| format!("{name} call"));
                let description = description.unwrap_or_else(|| title.clone());
                NormalizedEvent::ToolCallStart {
                    call,
                    name,
                    title,
                    description,
                    args,
                }
            }
            ProviderEventLine::ToolCallEnd { call }
            | ProviderEventLine::ToolCallResult { call } => NormalizedEvent::ToolCallEnd { call },
            ProviderEventLine::File {
                reference,
                name,
                size,
                mime_type,
            } => NormalizedEvent::File {
                reference,
                name,
                size,
                mime_type,
            },
        };
    }

    NormalizedEvent::Text {
        text: trimmed.to_string(),
    }
}

pub fn render_event_for_output(event: &NormalizedEvent) -> Option<String> {
    match event {
        NormalizedEvent::Text { text }
        | NormalizedEvent::Thinking { text }
        | NormalizedEvent::Service { text } => {
            if text.trim().is_empty() {
                None
            } else {
                Some(text.clone())
            }
        }
        NormalizedEvent::ToolCallStart { title, .. } => Some(title.clone()),
        NormalizedEvent::ToolCallEnd { .. } => None,
        NormalizedEvent::File { name, .. } => Some(name.clone()),
    }
}

pub fn as_message_json(output: &NormalizedOutput) -> Value {
    serde_json::json!({
        "type": "message",
        "message": output.text,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        NormalizedEvent, as_message_json, normalize_output_line, normalize_stdout,
        render_event_for_output,
    };

    #[test]
    fn normalizes_provider_output() {
        let output = normalize_stdout("hello\n");
        assert_eq!(output.text, "hello");
        assert_eq!(as_message_json(&output)["message"], "hello");
    }

    #[test]
    fn parses_structured_provider_events() {
        let tool_call = normalize_output_line(
            r#"{"type":"tool-call-start","call":"call-1","name":"shell","args":{"command":"ls"}}"#,
        );
        assert!(matches!(
            tool_call,
            NormalizedEvent::ToolCallStart {
                ref call,
                ref name,
                ref args,
                ..
            } if call == "call-1" && name == "shell" && args["command"] == json!("ls")
        ));

        let thinking = normalize_output_line(r#"{"type":"thinking","text":"planning"}"#);
        assert_eq!(
            render_event_for_output(&thinking),
            Some("planning".to_string())
        );

        let file = normalize_output_line(
            r#"{"type":"file","ref":"file-1","name":"demo.txt","size":4,"mimeType":"text/plain"}"#,
        );
        assert!(matches!(
            file,
            NormalizedEvent::File {
                ref reference,
                ref name,
                size,
                ref mime_type,
            } if reference == "file-1" && name == "demo.txt" && size == 4 && mime_type.as_deref() == Some("text/plain")
        ));
    }
}
