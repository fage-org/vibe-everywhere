use anyhow::Result;

use crate::{
    config::Config,
    persistence::{
        LocalSessionState, PersistenceError, list_local_sessions, resolve_local_session,
    },
};

pub fn list_resumable_sessions(
    config: &Config,
) -> Result<Vec<LocalSessionState>, PersistenceError> {
    list_local_sessions(config)
}

pub fn resolve_resumable_session(
    config: &Config,
    value: &str,
) -> Result<LocalSessionState, PersistenceError> {
    resolve_local_session(config, value)
}
