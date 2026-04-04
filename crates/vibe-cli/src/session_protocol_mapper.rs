use serde_json::Value;
use vibe_wire::{
    CreateEnvelopeOptions, SessionEvent, SessionFileEvent, SessionProtocolMessage, SessionRole,
    SessionServiceEvent, SessionTextEvent, SessionToolCallEndEvent, SessionToolCallStartEvent,
    SessionTurnEndEvent, SessionTurnEndStatus, SessionTurnStartEvent, create_envelope,
};

use crate::agent::adapters::{NormalizedEvent, as_message_json, normalize_stdout};

pub fn map_user_prompt(prompt: &str) -> Value {
    serde_json::json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": prompt,
        },
        "meta": {
            "sentFrom": "vibe",
        }
    })
}

pub fn map_agent_output(output: &str) -> Value {
    let normalized = normalize_stdout(output);
    serde_json::json!({
        "role": "agent",
        "content": {
            "type": "output",
            "data": as_message_json(&normalized),
        },
        "meta": {
            "sentFrom": "vibe",
        }
    })
}

pub fn map_turn_start(turn_id: &str) -> Value {
    session_protocol_message(
        SessionEvent::from(SessionTurnStartEvent::default()),
        Some(turn_id),
    )
}

pub fn map_turn_text(turn_id: &str, text: &str) -> Value {
    session_protocol_message(
        SessionEvent::from(SessionTextEvent::new(text.to_string())),
        Some(turn_id),
    )
}

pub fn map_normalized_event(turn_id: &str, event: &NormalizedEvent) -> Option<Value> {
    match event {
        NormalizedEvent::Text { text } => {
            if text.trim().is_empty() {
                None
            } else {
                Some(map_turn_text(turn_id, text))
            }
        }
        NormalizedEvent::Thinking { text } => Some(session_protocol_message(
            SessionEvent::from(SessionTextEvent {
                text: text.clone(),
                thinking: Some(true),
            }),
            Some(turn_id),
        )),
        NormalizedEvent::Service { text } => Some(session_protocol_message(
            SessionEvent::from(SessionServiceEvent { text: text.clone() }),
            Some(turn_id),
        )),
        NormalizedEvent::ToolCallStart {
            call,
            name,
            title,
            description,
            args,
        } => Some(session_protocol_message(
            SessionEvent::from(SessionToolCallStartEvent {
                call: call.clone(),
                name: name.clone(),
                title: title.clone(),
                description: description.clone(),
                args: args.clone(),
            }),
            Some(turn_id),
        )),
        NormalizedEvent::ToolCallEnd { call } => Some(session_protocol_message(
            SessionEvent::from(SessionToolCallEndEvent { call: call.clone() }),
            Some(turn_id),
        )),
        NormalizedEvent::File {
            reference,
            name,
            size,
            mime_type,
        } => Some(session_protocol_message(
            SessionEvent::from(SessionFileEvent {
                r#ref: reference.clone(),
                name: name.clone(),
                size: *size,
                mime_type: mime_type.clone(),
                image: None,
            }),
            Some(turn_id),
        )),
    }
}

pub fn map_turn_end(turn_id: &str, status: SessionTurnEndStatus) -> Value {
    session_protocol_message(
        SessionEvent::from(SessionTurnEndEvent { status }),
        Some(turn_id),
    )
}

fn session_protocol_message(event: SessionEvent, turn_id: Option<&str>) -> Value {
    serde_json::to_value(SessionProtocolMessage::new(
        create_envelope(
            SessionRole::Agent,
            event,
            CreateEnvelopeOptions {
                turn: turn_id.map(ToOwned::to_owned),
                ..CreateEnvelopeOptions::default()
            },
        )
        .expect("session protocol envelope should be valid"),
    ))
    .expect("session protocol message should serialize")
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashSet};

    use serde_json::Value;

    use super::{
        map_agent_output, map_normalized_event, map_turn_end, map_turn_start, map_turn_text,
        map_user_prompt,
    };
    use crate::agent::adapters::NormalizedEvent;
    use vibe_wire::SessionTurnEndStatus;

    fn fixture_value(name: &str) -> Value {
        let fixtures: Value = serde_json::from_str(include_str!(
            "../../vibe-wire/fixtures/session-envelopes.json"
        ))
        .unwrap();
        fixtures
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["name"] == name)
            .unwrap_or_else(|| panic!("missing fixture {name}"))["value"]
            .clone()
    }

    fn assert_matches_fixture(mapped: &Value, fixture_name: &str) {
        let fixture = fixture_value(fixture_name);
        assert_eq!(mapped["content"]["ev"], fixture["ev"]);
        assert_eq!(mapped["content"]["role"], fixture["role"]);
        if fixture.get("turn").is_some() {
            assert_eq!(mapped["content"]["turn"], fixture["turn"]);
        }
    }

    fn validate_sequence(events: &[Value]) -> Result<(), &'static str> {
        let mut in_turn = false;
        let mut open_tool_calls = HashSet::new();

        for event in events {
            let kind = event["content"]["ev"]["t"]
                .as_str()
                .ok_or("missing event type")?;
            match kind {
                "turn-start" => {
                    if in_turn {
                        return Err("duplicate turn-start");
                    }
                    in_turn = true;
                }
                "text" | "service" | "file" => {
                    if !in_turn {
                        return Err("content event before turn-start");
                    }
                }
                "tool-call-start" => {
                    if !in_turn {
                        return Err("tool-call-start before turn-start");
                    }
                    let call = event["content"]["ev"]["call"]
                        .as_str()
                        .ok_or("missing tool call id")?;
                    open_tool_calls.insert(call.to_string());
                }
                "tool-call-end" => {
                    if !in_turn {
                        return Err("tool-call-end before turn-start");
                    }
                    let call = event["content"]["ev"]["call"]
                        .as_str()
                        .ok_or("missing tool call id")?;
                    if !open_tool_calls.remove(call) {
                        return Err("tool-call-end without matching start");
                    }
                }
                "turn-end" => {
                    if !in_turn {
                        return Err("turn-end without turn-start");
                    }
                    if !open_tool_calls.is_empty() {
                        return Err("turn-end with open tool calls");
                    }
                    in_turn = false;
                }
                _ => {}
            }
        }

        if in_turn {
            return Err("turn not closed");
        }

        Ok(())
    }

    #[test]
    fn maps_user_prompt_to_legacy_message() {
        let value = map_user_prompt("hello");
        assert_eq!(value["role"], "user");
        assert_eq!(value["content"]["text"], "hello");
    }

    #[test]
    fn maps_agent_output_to_legacy_message() {
        let value = map_agent_output("done");
        assert_eq!(value["role"], "agent");
        assert_eq!(value["content"]["data"]["message"], "done");
    }

    #[test]
    fn maps_turn_lifecycle_to_session_protocol() {
        let start = map_turn_start("turn-1");
        let text = map_turn_text("turn-1", "thinking");
        let end = map_turn_end("turn-1", SessionTurnEndStatus::Completed);

        assert_eq!(start["role"], "session");
        assert_eq!(start["content"]["ev"]["t"], "turn-start");
        assert_eq!(text["content"]["ev"]["t"], "text");
        assert_eq!(text["content"]["turn"], "turn-1");
        assert_eq!(end["content"]["ev"]["t"], "turn-end");
        assert_eq!(end["content"]["ev"]["status"], "completed");
    }

    #[test]
    fn maps_structured_events_to_session_protocol() {
        let thinking = map_normalized_event(
            "turn-1",
            &NormalizedEvent::Thinking {
                text: "plan".into(),
            },
        )
        .unwrap();
        assert_eq!(thinking["content"]["ev"]["thinking"], true);

        let tool_start = map_normalized_event(
            "turn-1",
            &NormalizedEvent::ToolCallStart {
                call: "call-1".into(),
                name: "shell".into(),
                title: "Run shell".into(),
                description: "Run shell".into(),
                args: BTreeMap::new(),
            },
        )
        .unwrap();
        assert_eq!(tool_start["content"]["ev"]["t"], "tool-call-start");

        let tool_end = map_normalized_event(
            "turn-1",
            &NormalizedEvent::ToolCallEnd {
                call: "call-1".into(),
            },
        )
        .unwrap();
        assert_eq!(tool_end["content"]["ev"]["t"], "tool-call-end");

        let file = map_normalized_event(
            "turn-1",
            &NormalizedEvent::File {
                reference: "file-1".into(),
                name: "demo.txt".into(),
                size: 4,
                mime_type: Some("text/plain".into()),
            },
        )
        .unwrap();
        assert_eq!(file["content"]["ev"]["t"], "file");
        assert_eq!(file["content"]["ev"]["name"], "demo.txt");
    }

    #[test]
    fn mapped_events_match_wire_fixtures() {
        let turn_start = map_turn_start("turn-1");
        assert_matches_fixture(&turn_start, "session-turn-start-envelope");

        let thinking = map_normalized_event(
            "turn-1",
            &NormalizedEvent::Thinking {
                text: "thinking".into(),
            },
        )
        .unwrap();
        assert_matches_fixture(&thinking, "session-thinking-envelope");

        let service = map_normalized_event(
            "turn-1",
            &NormalizedEvent::Service {
                text: "**Service:** restarting MCP bridge".into(),
            },
        )
        .unwrap();
        assert_matches_fixture(&service, "session-service-envelope");

        let tool_start = map_normalized_event(
            "turn-1",
            &NormalizedEvent::ToolCallStart {
                call: "call-1".into(),
                name: "CodexBash".into(),
                title: "Run `ls`".into(),
                description: "Run `ls -la` in the repo root".into(),
                args: BTreeMap::from([(String::from("command"), Value::String("ls -la".into()))]),
            },
        )
        .unwrap();
        assert_matches_fixture(&tool_start, "session-tool-call-start-envelope");

        let tool_end = map_normalized_event(
            "turn-1",
            &NormalizedEvent::ToolCallEnd {
                call: "call-1".into(),
            },
        )
        .unwrap();
        assert_matches_fixture(&tool_end, "session-tool-call-end-envelope");

        let file = map_normalized_event(
            "turn-1",
            &NormalizedEvent::File {
                reference: "upload-1".into(),
                name: "report.txt".into(),
                size: 1024,
                mime_type: Some("text/plain".into()),
            },
        )
        .unwrap();
        assert_matches_fixture(&file, "session-file-envelope");

        let turn_end = map_turn_end("turn-1", SessionTurnEndStatus::Completed);
        assert_matches_fixture(&turn_end, "session-turn-end-envelope");
    }

    #[test]
    fn valid_sequence_passes_validation() {
        let sequence = vec![
            map_turn_start("turn-1"),
            map_normalized_event(
                "turn-1",
                &NormalizedEvent::ToolCallStart {
                    call: "call-1".into(),
                    name: "shell".into(),
                    title: "Run shell".into(),
                    description: "Run shell".into(),
                    args: BTreeMap::new(),
                },
            )
            .unwrap(),
            map_normalized_event(
                "turn-1",
                &NormalizedEvent::ToolCallEnd {
                    call: "call-1".into(),
                },
            )
            .unwrap(),
            map_turn_end("turn-1", SessionTurnEndStatus::Completed),
        ];
        assert!(validate_sequence(&sequence).is_ok());
    }

    #[test]
    fn invalid_sequence_rejects_unmatched_tool_end() {
        let sequence = vec![
            map_turn_start("turn-1"),
            map_normalized_event(
                "turn-1",
                &NormalizedEvent::ToolCallEnd {
                    call: "call-1".into(),
                },
            )
            .unwrap(),
            map_turn_end("turn-1", SessionTurnEndStatus::Completed),
        ];
        assert_eq!(
            validate_sequence(&sequence),
            Err("tool-call-end without matching start")
        );
    }
}
