use crate::persistence::LocalSessionState;

pub fn attachable_session_id(state: &LocalSessionState) -> &str {
    &state.provider_session_id
}

#[cfg(test)]
mod tests {
    use crate::persistence::LocalSessionState;

    use super::attachable_session_id;

    #[test]
    fn returns_provider_session_id() {
        let state = LocalSessionState {
            id: "local".into(),
            provider: "acp".into(),
            provider_session_id: "provider".into(),
            server_session_id: "server".into(),
            encryption_key: None,
            encryption_variant: "legacy".into(),
            tag: "tag".into(),
            working_dir: "/tmp".into(),
            created_at: 1,
            updated_at: 2,
        };
        assert_eq!(attachable_session_id(&state), "provider");
    }
}
