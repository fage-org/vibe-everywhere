use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use vibe_wire::SessionMessageContent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRecord {
    pub id: String,
    pub public_key: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub avatar: Option<Value>,
    pub github_profile: Option<Value>,
    pub settings: Option<String>,
    pub settings_version: u64,
    pub seq: u64,
    pub feed_seq: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageReportRecord {
    pub id: String,
    pub account_id: String,
    pub session_id: Option<String>,
    pub key: String,
    pub tokens: BTreeMap<String, u64>,
    pub cost: BTreeMap<String, f64>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvRecord {
    pub account_id: String,
    pub key: String,
    pub value: Option<String>,
    pub version: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushTokenRecord {
    pub id: String,
    pub account_id: String,
    pub token: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorTokenRecord {
    pub id: String,
    pub account_id: String,
    pub vendor: String,
    pub token: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubTokenRecord {
    pub id: String,
    pub account_id: String,
    pub github_user_id: String,
    pub token: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub id: String,
    pub account_id: String,
    pub header: String,
    pub header_version: u64,
    pub body: String,
    pub body_version: u64,
    pub data_encryption_key: String,
    pub seq: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessKeyRecord {
    pub account_id: String,
    pub session_id: String,
    pub machine_id: String,
    pub data: String,
    pub data_version: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRecord {
    pub account_id: String,
    pub friend_account_id: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipRecord {
    pub from_account_id: String,
    pub to_account_id: String,
    pub status: String,
    pub last_notified_at: Option<u64>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedPostRecord {
    pub id: String,
    pub account_id: String,
    pub repeat_key: Option<String>,
    pub body: Value,
    pub cursor: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalAuthRequestRecord {
    pub id: String,
    pub public_key: String,
    pub supports_v2: bool,
    pub response: Option<String>,
    pub response_account_id: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountAuthRequestRecord {
    pub id: String,
    pub public_key: String,
    pub response: Option<String>,
    pub response_account_id: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub account_id: String,
    pub tag: String,
    pub metadata: String,
    pub metadata_version: u64,
    pub agent_state: Option<String>,
    pub agent_state_version: u64,
    pub data_encryption_key: Option<String>,
    pub seq: u64,
    pub active: bool,
    pub active_at: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessageRecord {
    pub id: String,
    pub session_id: String,
    pub seq: u64,
    pub local_id: Option<String>,
    pub content: SessionMessageContent,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineRecord {
    pub id: String,
    pub account_id: String,
    pub metadata: String,
    pub metadata_version: u64,
    pub daemon_state: Option<String>,
    pub daemon_state_version: u64,
    pub data_encryption_key: Option<String>,
    pub seq: u64,
    pub active: bool,
    pub active_at: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone)]
pub enum CompareAndSwap<T> {
    Success(T),
    VersionMismatch {
        current_version: u64,
        current_value: T,
    },
    NotFound,
}

#[derive(Debug, Clone)]
pub enum ArtifactCreateOutcome {
    Created(ArtifactRecord),
    ExistingOwned(ArtifactRecord),
    ExistsOtherAccount,
}

#[derive(Debug, Default)]
pub struct DatabaseState {
    pub accounts: BTreeMap<String, AccountRecord>,
    pub account_ids_by_public_key: HashMap<String, String>,
    pub account_ids_by_username: HashMap<String, String>,
    pub terminal_auth_requests: HashMap<String, TerminalAuthRequestRecord>,
    pub account_auth_requests: HashMap<String, AccountAuthRequestRecord>,
    pub usage_reports: BTreeMap<String, UsageReportRecord>,
    pub kv: BTreeMap<(String, String), KvRecord>,
    pub push_tokens: BTreeMap<(String, String), PushTokenRecord>,
    pub vendor_tokens: BTreeMap<(String, String), VendorTokenRecord>,
    pub github_tokens: BTreeMap<String, GithubTokenRecord>,
    pub sessions: BTreeMap<String, SessionRecord>,
    pub session_ids_by_account_tag: HashMap<(String, String), String>,
    pub session_messages: BTreeMap<String, SessionMessageRecord>,
    pub session_message_ids_by_session: HashMap<String, Vec<String>>,
    pub session_message_ids_by_local_id: HashMap<(String, String), String>,
    pub machines: BTreeMap<(String, String), MachineRecord>,
    pub artifacts: BTreeMap<String, ArtifactRecord>,
    pub access_keys: BTreeMap<(String, String, String), AccessKeyRecord>,
    pub friends: BTreeMap<(String, String), FriendRecord>,
    pub relationships: BTreeMap<(String, String), RelationshipRecord>,
    pub feed_posts: BTreeMap<String, FeedPostRecord>,
}

#[derive(Debug, Clone, Default)]
pub struct Database {
    pub(super) inner: Arc<RwLock<DatabaseState>>,
}

impl Database {
    pub fn upsert_account_by_public_key(&self, public_key_hex: &str) -> AccountRecord {
        self.write(|state| {
            if let Some(id) = state.account_ids_by_public_key.get(public_key_hex).cloned() {
                let account = state.accounts.get_mut(&id).expect("account index drift");
                account.updated_at = now_ms();
                return account.clone();
            }

            let account = AccountRecord {
                id: new_id("acct"),
                public_key: public_key_hex.to_string(),
                first_name: None,
                last_name: None,
                username: None,
                avatar: None,
                github_profile: None,
                settings: None,
                settings_version: 0,
                seq: 0,
                feed_seq: 0,
                created_at: now_ms(),
                updated_at: now_ms(),
            };
            state
                .account_ids_by_public_key
                .insert(public_key_hex.to_string(), account.id.clone());
            state.accounts.insert(account.id.clone(), account.clone());
            account
        })
    }

    pub fn get_account(&self, account_id: &str) -> Option<AccountRecord> {
        self.read(|state| state.accounts.get(account_id).cloned())
    }

    pub fn put_terminal_auth_request(
        &self,
        public_key_hex: &str,
        supports_v2: bool,
    ) -> TerminalAuthRequestRecord {
        self.write(|state| {
            if let Some(existing) = state.terminal_auth_requests.get_mut(public_key_hex) {
                existing.updated_at = now_ms();
                return existing.clone();
            }

            let record = TerminalAuthRequestRecord {
                id: new_id("tar"),
                public_key: public_key_hex.to_string(),
                supports_v2,
                response: None,
                response_account_id: None,
                created_at: now_ms(),
                updated_at: now_ms(),
            };
            state
                .terminal_auth_requests
                .insert(public_key_hex.to_string(), record.clone());
            record
        })
    }

    pub fn get_terminal_auth_request(
        &self,
        public_key_hex: &str,
    ) -> Option<TerminalAuthRequestRecord> {
        self.read(|state| state.terminal_auth_requests.get(public_key_hex).cloned())
    }

    pub fn authorize_terminal_auth_request(
        &self,
        public_key_hex: &str,
        response: &str,
        account_id: &str,
    ) -> bool {
        self.write(|state| {
            let Some(record) = state.terminal_auth_requests.get_mut(public_key_hex) else {
                return false;
            };
            if record.response.is_none() {
                record.response = Some(response.to_string());
                record.response_account_id = Some(account_id.to_string());
                record.updated_at = now_ms();
            }
            true
        })
    }

    pub fn put_account_auth_request(&self, public_key_hex: &str) -> AccountAuthRequestRecord {
        self.write(|state| {
            if let Some(existing) = state.account_auth_requests.get_mut(public_key_hex) {
                existing.updated_at = now_ms();
                return existing.clone();
            }

            let record = AccountAuthRequestRecord {
                id: new_id("aar"),
                public_key: public_key_hex.to_string(),
                response: None,
                response_account_id: None,
                created_at: now_ms(),
                updated_at: now_ms(),
            };
            state
                .account_auth_requests
                .insert(public_key_hex.to_string(), record.clone());
            record
        })
    }

    pub fn get_account_auth_request(
        &self,
        public_key_hex: &str,
    ) -> Option<AccountAuthRequestRecord> {
        self.read(|state| state.account_auth_requests.get(public_key_hex).cloned())
    }

    pub fn authorize_account_auth_request(
        &self,
        public_key_hex: &str,
        response: &str,
        account_id: &str,
    ) -> bool {
        self.write(|state| {
            let Some(record) = state.account_auth_requests.get_mut(public_key_hex) else {
                return false;
            };
            if record.response.is_none() {
                record.response = Some(response.to_string());
                record.response_account_id = Some(account_id.to_string());
                record.updated_at = now_ms();
            }
            true
        })
    }

    pub fn allocate_account_seq(&self, account_id: &str) -> Option<u64> {
        self.write(|state| {
            let account = state.accounts.get_mut(account_id)?;
            account.seq += 1;
            account.updated_at = now_ms();
            Some(account.seq)
        })
    }

    pub fn create_artifact(
        &self,
        account_id: &str,
        id: String,
        header: String,
        body: String,
        data_encryption_key: String,
    ) -> ArtifactCreateOutcome {
        self.write(|state| {
            if let Some(existing) = state.artifacts.get(&id).cloned() {
                if existing.account_id == account_id {
                    return ArtifactCreateOutcome::ExistingOwned(existing);
                }
                return ArtifactCreateOutcome::ExistsOtherAccount;
            }

            let record = ArtifactRecord {
                id: id.clone(),
                account_id: account_id.to_string(),
                header,
                header_version: 1,
                body,
                body_version: 1,
                data_encryption_key,
                seq: 0,
                created_at: now_ms(),
                updated_at: now_ms(),
            };
            state.artifacts.insert(id, record.clone());
            ArtifactCreateOutcome::Created(record)
        })
    }

    pub fn create_or_load_session(
        &self,
        account_id: &str,
        tag: &str,
        metadata: &str,
        data_encryption_key: Option<String>,
    ) -> (SessionRecord, bool) {
        self.write(|state| {
            if let Some(session_id) = state
                .session_ids_by_account_tag
                .get(&(account_id.to_string(), tag.to_string()))
                .cloned()
            {
                let session = state
                    .sessions
                    .get(&session_id)
                    .expect("session index drift");
                return (session.clone(), false);
            }

            let record = SessionRecord {
                id: new_id("ses"),
                account_id: account_id.to_string(),
                tag: tag.to_string(),
                metadata: metadata.to_string(),
                metadata_version: 0,
                agent_state: None,
                agent_state_version: 0,
                data_encryption_key,
                seq: 0,
                active: true,
                active_at: now_ms(),
                created_at: now_ms(),
                updated_at: now_ms(),
            };
            state
                .session_ids_by_account_tag
                .insert((account_id.to_string(), tag.to_string()), record.id.clone());
            state.sessions.insert(record.id.clone(), record.clone());
            (record, true)
        })
    }

    pub fn list_sessions_by_updated_at(
        &self,
        account_id: &str,
        limit: usize,
    ) -> Vec<SessionRecord> {
        let mut sessions = self.read(|state| {
            state
                .sessions
                .values()
                .filter(|session| session.account_id == account_id)
                .cloned()
                .collect::<Vec<_>>()
        });
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        sessions.truncate(limit);
        sessions
    }

    pub fn list_active_sessions(
        &self,
        account_id: &str,
        limit: usize,
        active_after: u64,
    ) -> Vec<SessionRecord> {
        let mut sessions = self.read(|state| {
            state
                .sessions
                .values()
                .filter(|session| {
                    session.account_id == account_id
                        && session.active
                        && session.active_at > active_after
                })
                .cloned()
                .collect::<Vec<_>>()
        });
        sessions.sort_by(|a, b| b.active_at.cmp(&a.active_at));
        sessions.truncate(limit);
        sessions
    }

    pub fn page_sessions(
        &self,
        account_id: &str,
        cursor_lt_id: Option<&str>,
        changed_since: Option<u64>,
        limit: usize,
    ) -> (Vec<SessionRecord>, bool) {
        let mut sessions = self.read(|state| {
            state
                .sessions
                .values()
                .filter(|session| session.account_id == account_id)
                .filter(|session| changed_since.is_none_or(|ts| session.updated_at > ts))
                .filter(|session| cursor_lt_id.is_none_or(|cursor| session.id.as_str() < cursor))
                .cloned()
                .collect::<Vec<_>>()
        });
        sessions.sort_by(|a, b| b.id.cmp(&a.id));
        let has_next = sessions.len() > limit;
        if has_next {
            sessions.truncate(limit);
        }
        (sessions, has_next)
    }

    pub fn get_session_for_account(
        &self,
        account_id: &str,
        session_id: &str,
    ) -> Option<SessionRecord> {
        self.read(|state| {
            state
                .sessions
                .get(session_id)
                .filter(|session| session.account_id == account_id)
                .cloned()
        })
    }

    pub fn delete_session_for_account(
        &self,
        account_id: &str,
        session_id: &str,
    ) -> Option<SessionRecord> {
        self.write(|state| {
            let session = state.sessions.get(session_id)?.clone();
            if session.account_id != account_id {
                return None;
            }
            state
                .session_ids_by_account_tag
                .remove(&(account_id.to_string(), session.tag.clone()));
            state.sessions.remove(session_id);
            if let Some(message_ids) = state.session_message_ids_by_session.remove(session_id) {
                for message_id in message_ids {
                    if let Some(message) = state.session_messages.remove(&message_id)
                        && let Some(local_id) = message.local_id
                    {
                        state
                            .session_message_ids_by_local_id
                            .remove(&(session_id.to_string(), local_id));
                    }
                }
            }
            Some(session)
        })
    }

    pub fn allocate_session_seq(&self, session_id: &str) -> Option<u64> {
        self.write(|state| {
            let session = state.sessions.get_mut(session_id)?;
            session.seq += 1;
            session.updated_at = now_ms();
            Some(session.seq)
        })
    }

    pub fn allocate_session_seq_batch(&self, session_id: &str, count: usize) -> Option<Vec<u64>> {
        if count == 0 {
            return Some(Vec::new());
        }
        self.write(|state| {
            let session = state.sessions.get_mut(session_id)?;
            let start = session.seq + 1;
            session.seq += count as u64;
            session.updated_at = now_ms();
            let end = session.seq;
            Some((start..=end).collect())
        })
    }

    pub fn list_session_messages_desc(
        &self,
        session_id: &str,
        limit: usize,
    ) -> Vec<SessionMessageRecord> {
        let mut messages = self.read(|state| {
            state
                .session_message_ids_by_session
                .get(session_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|id| state.session_messages.get(&id).cloned())
                .collect::<Vec<_>>()
        });
        messages.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        messages.truncate(limit);
        messages
    }

    pub fn page_session_messages(
        &self,
        session_id: &str,
        after_seq: u64,
        limit: usize,
    ) -> (Vec<SessionMessageRecord>, bool) {
        let mut messages = self.read(|state| {
            state
                .session_message_ids_by_session
                .get(session_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|id| state.session_messages.get(&id).cloned())
                .filter(|message| message.seq > after_seq)
                .collect::<Vec<_>>()
        });
        messages.sort_by(|a, b| a.seq.cmp(&b.seq));
        let has_more = messages.len() > limit;
        if has_more {
            messages.truncate(limit);
        }
        (messages, has_more)
    }

    pub fn append_message(
        &self,
        account_id: &str,
        session_id: &str,
        content: SessionMessageContent,
        local_id: Option<String>,
    ) -> Option<(SessionMessageRecord, bool)> {
        self.write(|state| {
            let session = state.sessions.get(session_id)?;
            if session.account_id != account_id {
                return None;
            }

            if let Some(local_id) = local_id.as_ref()
                && let Some(existing_id) = state
                    .session_message_ids_by_local_id
                    .get(&(session_id.to_string(), local_id.clone()))
                    .cloned()
            {
                let existing = state.session_messages.get(&existing_id)?.clone();
                return Some((existing, false));
            }

            let timestamp = now_ms();
            let seq = {
                let session = state.sessions.get_mut(session_id)?;
                session.seq += 1;
                session.updated_at = timestamp;
                session.seq
            };
            let record = SessionMessageRecord {
                id: new_id("msg"),
                session_id: session_id.to_string(),
                seq,
                local_id: local_id.clone(),
                content,
                created_at: timestamp,
                updated_at: timestamp,
            };
            state
                .session_messages
                .insert(record.id.clone(), record.clone());
            state
                .session_message_ids_by_session
                .entry(session_id.to_string())
                .or_default()
                .push(record.id.clone());
            if let Some(local_id) = local_id {
                state
                    .session_message_ids_by_local_id
                    .insert((session_id.to_string(), local_id), record.id.clone());
            }
            Some((record, true))
        })
    }

    pub fn append_messages_idempotent(
        &self,
        account_id: &str,
        session_id: &str,
        messages: Vec<(String, String)>,
    ) -> Option<(Vec<SessionMessageRecord>, Vec<SessionMessageRecord>)> {
        self.write(|state| {
            let session = state.sessions.get(session_id)?;
            if session.account_id != account_id {
                return None;
            }

            let mut unique = Vec::new();
            let mut seen = HashMap::<String, String>::new();
            for (content, local_id) in messages {
                seen.entry(local_id.clone())
                    .or_insert_with(|| content.clone());
                if !unique.iter().any(|(_, lid)| lid == &local_id) {
                    unique.push((content, local_id));
                }
            }

            let mut existing = Vec::new();
            let mut created = Vec::new();
            let mut next_seq = state.sessions.get(session_id)?.seq;
            for (content, local_id) in unique {
                if let Some(existing_id) = state
                    .session_message_ids_by_local_id
                    .get(&(session_id.to_string(), local_id.clone()))
                    .cloned()
                {
                    if let Some(existing_message) =
                        state.session_messages.get(&existing_id).cloned()
                    {
                        existing.push(existing_message);
                    }
                    continue;
                }

                next_seq += 1;
                let record = SessionMessageRecord {
                    id: new_id("msg"),
                    session_id: session_id.to_string(),
                    seq: next_seq,
                    local_id: Some(local_id.clone()),
                    content: SessionMessageContent::new(content),
                    created_at: now_ms(),
                    updated_at: now_ms(),
                };
                state
                    .session_messages
                    .insert(record.id.clone(), record.clone());
                state
                    .session_message_ids_by_session
                    .entry(session_id.to_string())
                    .or_default()
                    .push(record.id.clone());
                state
                    .session_message_ids_by_local_id
                    .insert((session_id.to_string(), local_id), record.id.clone());
                created.push(record);
            }
            if let Some(session) = state.sessions.get_mut(session_id) {
                session.seq = next_seq;
                if !created.is_empty() {
                    session.updated_at = now_ms();
                }
            }

            let mut all = existing.clone();
            all.extend(created.clone());
            all.sort_by(|a, b| a.seq.cmp(&b.seq));
            Some((all, created))
        })
    }

    pub fn update_session_metadata(
        &self,
        account_id: &str,
        session_id: &str,
        expected_version: u64,
        metadata: &str,
    ) -> CompareAndSwap<String> {
        self.write(|state| {
            let Some(current) = state.sessions.get(session_id).cloned() else {
                return CompareAndSwap::NotFound;
            };
            if current.account_id != account_id {
                return CompareAndSwap::NotFound;
            }
            if current.metadata_version != expected_version {
                return CompareAndSwap::VersionMismatch {
                    current_version: current.metadata_version,
                    current_value: current.metadata,
                };
            }

            let session = state.sessions.get_mut(session_id).expect("session missing");
            session.metadata = metadata.to_string();
            session.metadata_version += 1;
            session.updated_at = now_ms();
            CompareAndSwap::Success(metadata.to_string())
        })
    }

    pub fn update_session_agent_state(
        &self,
        account_id: &str,
        session_id: &str,
        expected_version: u64,
        agent_state: Option<String>,
    ) -> CompareAndSwap<Option<String>> {
        self.write(|state| {
            let Some(current) = state.sessions.get(session_id).cloned() else {
                return CompareAndSwap::NotFound;
            };
            if current.account_id != account_id {
                return CompareAndSwap::NotFound;
            }
            if current.agent_state_version != expected_version {
                return CompareAndSwap::VersionMismatch {
                    current_version: current.agent_state_version,
                    current_value: current.agent_state,
                };
            }

            let session = state.sessions.get_mut(session_id).expect("session missing");
            session.agent_state = agent_state.clone();
            session.agent_state_version += 1;
            session.updated_at = now_ms();
            CompareAndSwap::Success(agent_state)
        })
    }

    pub fn create_or_load_machine(
        &self,
        account_id: &str,
        machine_id: &str,
        metadata: &str,
        daemon_state: Option<String>,
        data_encryption_key: Option<String>,
    ) -> (MachineRecord, bool) {
        self.write(|state| {
            let key = (account_id.to_string(), machine_id.to_string());
            if let Some(machine) = state.machines.get(&key) {
                return (machine.clone(), false);
            }

            let record = MachineRecord {
                id: machine_id.to_string(),
                account_id: account_id.to_string(),
                metadata: metadata.to_string(),
                metadata_version: 1,
                daemon_state: daemon_state.clone(),
                daemon_state_version: if daemon_state.is_some() { 1 } else { 0 },
                data_encryption_key,
                seq: 0,
                active: false,
                active_at: now_ms(),
                created_at: now_ms(),
                updated_at: now_ms(),
            };
            state.machines.insert(key, record.clone());
            (record, true)
        })
    }

    pub fn list_machines(&self, account_id: &str) -> Vec<MachineRecord> {
        let mut machines = self.read(|state| {
            state
                .machines
                .values()
                .filter(|machine| machine.account_id == account_id)
                .cloned()
                .collect::<Vec<_>>()
        });
        machines.sort_by(|a, b| b.active_at.cmp(&a.active_at));
        machines
    }

    pub fn get_machine_for_account(
        &self,
        account_id: &str,
        machine_id: &str,
    ) -> Option<MachineRecord> {
        self.read(|state| {
            state
                .machines
                .get(&(account_id.to_string(), machine_id.to_string()))
                .cloned()
        })
    }

    pub fn update_machine_metadata(
        &self,
        account_id: &str,
        machine_id: &str,
        expected_version: u64,
        metadata: &str,
    ) -> CompareAndSwap<String> {
        self.write(|state| {
            let key = (account_id.to_string(), machine_id.to_string());
            let Some(current) = state.machines.get(&key).cloned() else {
                return CompareAndSwap::NotFound;
            };
            if current.metadata_version != expected_version {
                return CompareAndSwap::VersionMismatch {
                    current_version: current.metadata_version,
                    current_value: current.metadata,
                };
            }

            let machine = state.machines.get_mut(&key).expect("machine missing");
            machine.metadata = metadata.to_string();
            machine.metadata_version += 1;
            machine.updated_at = now_ms();
            CompareAndSwap::Success(metadata.to_string())
        })
    }

    pub fn update_machine_daemon_state(
        &self,
        account_id: &str,
        machine_id: &str,
        expected_version: u64,
        daemon_state: String,
    ) -> CompareAndSwap<Option<String>> {
        self.write(|state| {
            let key = (account_id.to_string(), machine_id.to_string());
            let Some(current) = state.machines.get(&key).cloned() else {
                return CompareAndSwap::NotFound;
            };
            if current.daemon_state_version != expected_version {
                return CompareAndSwap::VersionMismatch {
                    current_version: current.daemon_state_version,
                    current_value: current.daemon_state,
                };
            }

            let machine = state.machines.get_mut(&key).expect("machine missing");
            machine.daemon_state = Some(daemon_state.clone());
            machine.daemon_state_version += 1;
            machine.active = true;
            machine.active_at = now_ms();
            machine.updated_at = now_ms();
            CompareAndSwap::Success(Some(daemon_state))
        })
    }

    pub fn set_session_presence(
        &self,
        account_id: &str,
        session_id: &str,
        active: bool,
        active_at: u64,
    ) -> Option<(SessionRecord, bool)> {
        self.write(|state| {
            let session = state.sessions.get_mut(session_id)?;
            if session.account_id != account_id {
                return None;
            }
            if active_at < session.active_at {
                return Some((session.clone(), false));
            }
            let transitioned = session.active != active;
            let next_active_at = session.active_at.max(active_at);
            session.active = active;
            session.active_at = next_active_at;
            session.updated_at = now_ms();
            Some((session.clone(), transitioned))
        })
    }

    pub fn set_machine_presence(
        &self,
        account_id: &str,
        machine_id: &str,
        active: bool,
        active_at: u64,
    ) -> Option<(MachineRecord, bool)> {
        self.write(|state| {
            let machine = state
                .machines
                .get_mut(&(account_id.to_string(), machine_id.to_string()))?;
            if active_at < machine.active_at {
                return Some((machine.clone(), false));
            }
            let transitioned = machine.active != active;
            let next_active_at = machine.active_at.max(active_at);
            machine.active = active;
            machine.active_at = next_active_at;
            machine.updated_at = now_ms();
            Some((machine.clone(), transitioned))
        })
    }

    pub fn timed_out_sessions(&self, cutoff: u64) -> Vec<SessionRecord> {
        self.read(|state| {
            state
                .sessions
                .values()
                .filter(|session| session.active && session.active_at <= cutoff)
                .cloned()
                .collect()
        })
    }

    pub fn timed_out_machines(&self, cutoff: u64) -> Vec<MachineRecord> {
        self.read(|state| {
            state
                .machines
                .values()
                .filter(|machine| machine.active && machine.active_at <= cutoff)
                .cloned()
                .collect()
        })
    }

    pub fn read<R>(&self, f: impl FnOnce(&DatabaseState) -> R) -> R {
        let guard = self.inner.read();
        f(&guard)
    }

    pub fn write<R>(&self, f: impl FnOnce(&mut DatabaseState) -> R) -> R {
        let mut guard = self.inner.write();
        f(&mut guard)
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_millis() as u64
}

fn new_id(prefix: &str) -> String {
    format!("{prefix}_{}", uuid::Uuid::now_v7())
}

#[cfg(test)]
mod tests {
    use vibe_wire::SessionMessageContent;

    use super::{ArtifactCreateOutcome, Database};

    #[test]
    fn sequence_allocation_is_monotonic() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("abcd");
        assert_eq!(db.allocate_account_seq(&account.id), Some(1));
        assert_eq!(db.allocate_account_seq(&account.id), Some(2));
    }

    #[test]
    fn batch_message_insert_is_idempotent_by_local_id() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("pk");
        let (session, _) = db.create_or_load_session(&account.id, "tag", "m", None);

        let (all, created) = db
            .append_messages_idempotent(
                &account.id,
                &session.id,
                vec![
                    ("a".into(), "local-1".into()),
                    ("b".into(), "local-1".into()),
                    ("c".into(), "local-2".into()),
                ],
            )
            .unwrap();

        assert_eq!(all.len(), 2);
        assert_eq!(created.len(), 2);

        let (existing, created_again) = db
            .append_messages_idempotent(
                &account.id,
                &session.id,
                vec![("a".into(), "local-1".into())],
            )
            .unwrap();
        assert_eq!(existing.len(), 1);
        assert!(created_again.is_empty());
    }

    #[test]
    fn append_message_deduplicates_local_id() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("pk");
        let (session, _) = db.create_or_load_session(&account.id, "tag", "m", None);
        let content = SessionMessageContent::new("cipher");
        let (_, created) = db
            .append_message(
                &account.id,
                &session.id,
                content.clone(),
                Some("loc-1".into()),
            )
            .unwrap();
        assert!(created);
        let (_, created_again) = db
            .append_message(&account.id, &session.id, content, Some("loc-1".into()))
            .unwrap();
        assert!(!created_again);
    }

    #[test]
    fn append_message_updates_session_timestamp() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("pk");
        let (session, _) = db.create_or_load_session(&account.id, "tag", "m", None);

        db.write(|state| {
            let session = state.sessions.get_mut(&session.id).unwrap();
            session.updated_at = 10;
        });

        db.append_message(
            &account.id,
            &session.id,
            SessionMessageContent::new("cipher"),
            Some("loc-1".into()),
        )
        .unwrap();

        let updated = db
            .get_session_for_account(&account.id, &session.id)
            .unwrap();
        assert!(updated.updated_at > 10);
    }

    #[test]
    fn bulk_message_insert_updates_session_timestamp_when_new_messages_are_created() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("pk");
        let (session, _) = db.create_or_load_session(&account.id, "tag", "m", None);

        db.write(|state| {
            let session = state.sessions.get_mut(&session.id).unwrap();
            session.updated_at = 10;
        });

        db.append_messages_idempotent(
            &account.id,
            &session.id,
            vec![("cipher".into(), "loc-1".into())],
        )
        .unwrap();

        let updated = db
            .get_session_for_account(&account.id, &session.id)
            .unwrap();
        assert!(updated.updated_at > 10);
    }

    #[test]
    fn machine_records_are_isolated_by_account_and_id() {
        let db = Database::default();
        let account_a = db.upsert_account_by_public_key("pk-a");
        let account_b = db.upsert_account_by_public_key("pk-b");

        let (machine_a, created_a) =
            db.create_or_load_machine(&account_a.id, "machine-1", "meta-a", None, None);
        let (machine_b, created_b) =
            db.create_or_load_machine(&account_b.id, "machine-1", "meta-b", None, None);

        assert!(created_a);
        assert!(created_b);
        assert_eq!(machine_a.id, "machine-1");
        assert_eq!(machine_b.id, "machine-1");

        assert_eq!(
            db.get_machine_for_account(&account_a.id, "machine-1")
                .unwrap()
                .metadata,
            "meta-a"
        );
        assert_eq!(
            db.get_machine_for_account(&account_b.id, "machine-1")
                .unwrap()
                .metadata,
            "meta-b"
        );
    }

    #[test]
    fn create_artifact_preserves_idempotent_and_conflict_semantics() {
        let db = Database::default();
        let account_a = db.upsert_account_by_public_key("pk-a");
        let account_b = db.upsert_account_by_public_key("pk-b");
        let artifact_id = uuid::Uuid::now_v7().to_string();

        let created = db.create_artifact(
            &account_a.id,
            artifact_id.clone(),
            "header-a".into(),
            "body-a".into(),
            "dek-a".into(),
        );
        let same_account = db.create_artifact(
            &account_a.id,
            artifact_id.clone(),
            "ignored".into(),
            "ignored".into(),
            "ignored".into(),
        );
        let other_account = db.create_artifact(
            &account_b.id,
            artifact_id.clone(),
            "header-b".into(),
            "body-b".into(),
            "dek-b".into(),
        );

        match created {
            ArtifactCreateOutcome::Created(record) => {
                assert_eq!(record.header, "header-a");
                assert_eq!(record.body, "body-a");
                assert_eq!(record.data_encryption_key, "dek-a");
            }
            other => panic!("expected create outcome, got {other:?}"),
        }

        match same_account {
            ArtifactCreateOutcome::ExistingOwned(record) => {
                assert_eq!(record.header, "header-a");
                assert_eq!(record.body, "body-a");
                assert_eq!(record.data_encryption_key, "dek-a");
            }
            other => panic!("expected same-account idempotent outcome, got {other:?}"),
        }

        assert!(matches!(
            other_account,
            ArtifactCreateOutcome::ExistsOtherAccount
        ));
    }
}
