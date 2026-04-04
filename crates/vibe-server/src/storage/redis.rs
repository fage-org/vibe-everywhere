use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearerTokenRecord {
    pub user_id: String,
    pub extras: Option<Value>,
}

#[derive(Debug, Clone)]
struct Entry {
    value: Value,
    expires_at: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct PresenceCacheEntry {
    pub valid_until: u64,
    pub last_update_sent: u64,
    pub pending_update: Option<u64>,
    pub active: bool,
}

#[derive(Debug, Default)]
pub struct PresenceCacheState {
    pub sessions: HashMap<(String, String), PresenceCacheEntry>,
    pub machines: HashMap<(String, String), PresenceCacheEntry>,
}

#[derive(Debug, Default)]
struct RedisState {
    kv: HashMap<String, Entry>,
    presence: PresenceCacheState,
}

#[derive(Debug, Clone, Default)]
pub struct RedisStore {
    inner: Arc<RwLock<RedisState>>,
}

impl RedisStore {
    pub fn cache_bearer_token(&self, token: &str, record: BearerTokenRecord) {
        self.set_json(
            token,
            serde_json::to_value(record).expect("token record serializable"),
            None,
        );
    }

    pub fn get_bearer_token(&self, token: &str) -> Option<BearerTokenRecord> {
        serde_json::from_value(self.get_json(token)?).ok()
    }

    pub fn set_json(&self, key: &str, value: Value, ttl_ms: Option<u64>) {
        let expires_at = ttl_ms.map(|ttl| now_ms().saturating_add(ttl));
        self.inner
            .write()
            .kv
            .insert(key.to_string(), Entry { value, expires_at });
    }

    pub fn get_json(&self, key: &str) -> Option<Value> {
        let mut guard = self.inner.write();
        let entry = guard.kv.get(key)?.clone();
        if entry
            .expires_at
            .is_some_and(|deadline| deadline <= now_ms())
        {
            guard.kv.remove(key);
            return None;
        }
        Some(entry.value)
    }

    pub fn read_presence_state<R>(&self, f: impl FnOnce(&PresenceCacheState) -> R) -> R {
        let guard = self.inner.read();
        f(&guard.presence)
    }

    pub fn write_presence_state<R>(&self, f: impl FnOnce(&mut PresenceCacheState) -> R) -> R {
        let mut guard = self.inner.write();
        f(&mut guard.presence)
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::{BearerTokenRecord, PresenceCacheEntry, RedisStore};

    #[test]
    fn token_round_trip_works() {
        let store = RedisStore::default();
        store.cache_bearer_token(
            "token",
            BearerTokenRecord {
                user_id: "acct".into(),
                extras: None,
            },
        );

        assert_eq!(store.get_bearer_token("token").unwrap().user_id, "acct");
    }

    #[test]
    fn ttl_expiry_removes_value() {
        let store = RedisStore::default();
        store.set_json("k", serde_json::json!({"x": 1}), Some(0));
        assert!(store.get_json("k").is_none());
    }

    #[test]
    fn presence_state_round_trip_works() {
        let store = RedisStore::default();
        store.write_presence_state(|state| {
            state.sessions.insert(
                ("acct".into(), "session-1".into()),
                PresenceCacheEntry {
                    valid_until: 10,
                    last_update_sent: 20,
                    pending_update: Some(30),
                    active: true,
                },
            );
        });

        let entry = store.read_presence_state(|state| {
            state
                .sessions
                .get(&("acct".into(), "session-1".into()))
                .cloned()
        });
        assert_eq!(entry.unwrap().pending_update, Some(30));
    }
}
