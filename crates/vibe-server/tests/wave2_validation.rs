use serde_json::Value;
use vibe_server::{
    config::Config,
    context::AppContext,
    events::{
        DurableUpdateContainer, build_new_message_update, build_update_machine_update,
        build_update_session_update,
    },
    machines::MachinesService,
    sessions::SessionsService,
    storage::db::{CompareAndSwap, SessionMessageRecord},
};
use vibe_wire::{
    SessionMessageContent, Update, VersionedEncryptedValue, VersionedNullableEncryptedValue,
    update_container_fixtures,
};

fn test_context() -> AppContext {
    AppContext::new(Config {
        host: "127.0.0.1".parse().unwrap(),
        port: 3005,
        master_secret: "secret".into(),
        ios_up_to_date: ">=1.4.1".into(),
        android_up_to_date: ">=1.4.1".into(),
        ios_store_url: "ios-store".into(),
        android_store_url: "android-store".into(),
        webapp_url: "https://app.vibe.engineering".into(),
    })
}

fn fixture(name: &str) -> Value {
    update_container_fixtures()
        .into_iter()
        .find(|fixture| fixture.name == name)
        .expect("fixture should exist")
        .value
}

#[test]
fn vibe_wire_update_fixtures_round_trip_through_server_container_types() {
    for fixture in update_container_fixtures() {
        let container: DurableUpdateContainer =
            serde_json::from_value(fixture.value.clone()).expect("server container should decode");
        assert_eq!(serde_json::to_value(&container).unwrap(), fixture.value);

        let update: Update =
            serde_json::from_value(fixture.value.clone()).expect("wire update should decode");
        assert_eq!(serde_json::to_value(&update).unwrap(), fixture.value);
    }
}

#[test]
fn server_core_update_builders_match_vibe_wire_fixture_shapes() {
    let message = SessionMessageRecord {
        id: "msg-1".into(),
        session_id: "session-1".into(),
        seq: 10,
        local_id: None,
        content: SessionMessageContent::new("ZmFrZS1lbmNyeXB0ZWQ="),
        created_at: 123,
        updated_at: 124,
    };
    let mut new_message = build_new_message_update("session-1", &message, 1);
    new_message.id = "upd-1".into();
    new_message.created_at = 1;
    assert_eq!(
        serde_json::to_value(&new_message).unwrap(),
        fixture("update-container-new-message")
    );

    let mut update_session = build_update_session_update(
        "session-1",
        3,
        Some(VersionedEncryptedValue {
            version: 2,
            value: "abc".into(),
        }),
        Some(VersionedNullableEncryptedValue {
            version: 3,
            value: None,
        }),
    );
    update_session.id = "upd-3".into();
    update_session.created_at = 3;
    assert_eq!(
        serde_json::to_value(&update_session).unwrap(),
        fixture("update-container-session-versioned-fields")
    );

    let mut update_machine =
        build_update_machine_update("machine-2", 5, None, None, Some(true), Some(12_345));
    update_machine.id = "upd-5".into();
    update_machine.created_at = 5;
    assert_eq!(
        serde_json::to_value(&update_machine).unwrap(),
        fixture("update-container-machine-presence-only")
    );
}

#[tokio::test]
async fn wave2_storage_flow_preserves_encrypted_fields_and_monotonic_sequences() {
    let ctx = test_context();
    let account = ctx.db().upsert_account_by_public_key("pk");
    let sessions = SessionsService::new(ctx.clone());
    let machines = MachinesService::new(ctx.clone());

    let session = sessions
        .create_or_load(
            &account.id,
            "tag-1",
            "session-meta-cipher",
            Some("session-key-cipher".into()),
        )
        .unwrap();
    sessions
        .append_single_message(
            &account.id,
            &session.id,
            "message-cipher".into(),
            Some("local-1".into()),
            None,
        )
        .unwrap();
    assert!(matches!(
        sessions
            .update_metadata(&account.id, &session.id, 0, "session-meta-next".into())
            .unwrap(),
        CompareAndSwap::Success(value) if value == "session-meta-next"
    ));
    assert!(matches!(
        sessions
            .update_agent_state(&account.id, &session.id, 0, Some("agent-state-cipher".into()))
            .unwrap(),
        CompareAndSwap::Success(Some(value)) if value == "agent-state-cipher"
    ));

    let machine = machines
        .create_or_load(
            &account.id,
            "machine-1",
            "machine-meta-cipher",
            Some("daemon-state-cipher-1".into()),
            Some("machine-key-cipher".into()),
        )
        .unwrap();
    assert!(matches!(
        machines
            .update_metadata(&account.id, &machine.id, 1, "machine-meta-next".into())
            .unwrap(),
        CompareAndSwap::Success(value) if value == "machine-meta-next"
    ));
    assert!(matches!(
        machines
            .update_daemon_state(&account.id, &machine.id, 1, "daemon-state-cipher-2".into())
            .unwrap(),
        CompareAndSwap::Success(Some(value)) if value == "daemon-state-cipher-2"
    ));

    let stored_session = ctx
        .db()
        .get_session_for_account(&account.id, &session.id)
        .expect("session should exist");
    assert_eq!(stored_session.metadata, "session-meta-next");
    assert_eq!(stored_session.metadata_version, 1);
    assert_eq!(
        stored_session.agent_state.as_deref(),
        Some("agent-state-cipher")
    );
    assert_eq!(stored_session.agent_state_version, 1);
    assert_eq!(
        stored_session.data_encryption_key.as_deref(),
        Some("session-key-cipher")
    );

    let (stored_messages, has_more) = ctx.db().page_session_messages(&session.id, 0, 10);
    assert!(!has_more);
    assert_eq!(stored_messages.len(), 1);
    assert_eq!(stored_messages[0].content.ciphertext, "message-cipher");
    assert_eq!(stored_messages[0].local_id.as_deref(), Some("local-1"));

    let stored_machine = ctx
        .db()
        .get_machine_for_account(&account.id, &machine.id)
        .expect("machine should exist");
    assert_eq!(stored_machine.metadata, "machine-meta-next");
    assert_eq!(stored_machine.metadata_version, 2);
    assert_eq!(
        stored_machine.daemon_state.as_deref(),
        Some("daemon-state-cipher-2")
    );
    assert_eq!(stored_machine.daemon_state_version, 2);
    assert_eq!(
        stored_machine.data_encryption_key.as_deref(),
        Some("machine-key-cipher")
    );

    let account_record = ctx
        .db()
        .get_account(&account.id)
        .expect("account should exist");
    assert_eq!(account_record.seq, 8);
}
