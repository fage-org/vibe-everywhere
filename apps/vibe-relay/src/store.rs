use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path as FsPath};
use vibe_core::{
    AuditRecord, ConversationInputRequest, ConversationRecord, DeviceRecord, MembershipRecord,
    PortForwardRecord, ShellInputRecord, ShellOutputChunk, ShellSessionRecord, TaskEvent,
    TaskRecord, TenantRecord, UserRecord,
};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub(crate) struct RelayStore {
    pub(crate) tenants: HashMap<String, TenantRecord>,
    pub(crate) users: HashMap<String, UserRecord>,
    pub(crate) memberships: Vec<MembershipRecord>,
    pub(crate) audit_records: Vec<AuditRecord>,
    pub(crate) device_credentials: HashMap<String, DeviceCredentialRecord>,
    pub(crate) devices: HashMap<String, DeviceRecord>,
    pub(crate) conversations: HashMap<String, ConversationEntry>,
    pub(crate) input_requests: HashMap<String, ConversationInputRequest>,
    pub(crate) tasks: HashMap<String, TaskEntry>,
    pub(crate) shell_sessions: HashMap<String, ShellSessionEntry>,
    pub(crate) port_forwards: HashMap<String, PortForwardEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct DeviceCredentialRecord {
    pub(crate) device_id: String,
    pub(crate) token: String,
    pub(crate) issued_at_epoch_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct TaskEntry {
    pub(crate) record: TaskRecord,
    pub(crate) events: Vec<TaskEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ConversationEntry {
    pub(crate) record: ConversationRecord,
    pub(crate) task_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ShellSessionEntry {
    pub(crate) record: ShellSessionRecord,
    pub(crate) inputs: Vec<ShellInputRecord>,
    pub(crate) outputs: Vec<ShellOutputChunk>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PortForwardEntry {
    pub(crate) record: PortForwardRecord,
}

pub(crate) fn load_relay_store(path: &FsPath) -> Result<RelayStore, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(RelayStore::default());
    }

    let bytes = std::fs::read(path)?;
    let store = serde_json::from_slice::<RelayStore>(&bytes)?;
    Ok(store)
}

pub(crate) fn persist_relay_store(
    path: &FsPath,
    store: &RelayStore,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let encoded = serde_json::to_vec_pretty(store)?;
    std::fs::write(path, encoded)?;
    Ok(())
}
