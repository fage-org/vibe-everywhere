use serde::{Deserialize, Serialize};
use vibe_wire::CoreUpdateBody;

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t")]
pub enum LateDurableUpdate {
    #[serde(rename = "new-session")]
    NewSession {
        id: String,
        seq: u64,
        metadata: String,
        #[serde(rename = "metadataVersion")]
        metadata_version: u64,
        #[serde(rename = "agentState")]
        agent_state: Option<String>,
        #[serde(rename = "agentStateVersion")]
        agent_state_version: u64,
        #[serde(rename = "dataEncryptionKey")]
        data_encryption_key: Option<String>,
        active: bool,
        #[serde(rename = "activeAt")]
        active_at: u64,
        #[serde(rename = "createdAt")]
        created_at: u64,
        #[serde(rename = "updatedAt")]
        updated_at: u64,
    },
    #[serde(rename = "delete-session")]
    DeleteSession { sid: String },
    #[serde(rename = "new-machine")]
    NewMachine {
        #[serde(rename = "machineId")]
        machine_id: String,
        seq: u64,
        metadata: String,
        #[serde(rename = "metadataVersion")]
        metadata_version: u64,
        #[serde(rename = "daemonState")]
        daemon_state: Option<String>,
        #[serde(rename = "daemonStateVersion")]
        daemon_state_version: u64,
        #[serde(rename = "dataEncryptionKey")]
        data_encryption_key: Option<String>,
        active: bool,
        #[serde(rename = "activeAt")]
        active_at: u64,
        #[serde(rename = "createdAt")]
        created_at: u64,
        #[serde(rename = "updatedAt")]
        updated_at: u64,
    },
    #[serde(rename = "update-account")]
    UpdateAccount {
        id: String,
        #[serde(flatten, default)]
        changes: BTreeMap<String, serde_json::Value>,
    },
    #[serde(rename = "new-artifact")]
    NewArtifact {
        #[serde(rename = "artifactId")]
        artifact_id: String,
        seq: u64,
        header: String,
        #[serde(rename = "headerVersion")]
        header_version: u64,
        body: String,
        #[serde(rename = "bodyVersion")]
        body_version: u64,
        #[serde(rename = "dataEncryptionKey")]
        data_encryption_key: String,
        #[serde(rename = "createdAt")]
        created_at: u64,
        #[serde(rename = "updatedAt")]
        updated_at: u64,
    },
    #[serde(rename = "update-artifact")]
    UpdateArtifact {
        #[serde(rename = "artifactId")]
        artifact_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        header: Option<LateVersionedValue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<LateVersionedValue>,
    },
    #[serde(rename = "delete-artifact")]
    DeleteArtifact {
        #[serde(rename = "artifactId")]
        artifact_id: String,
    },
    #[serde(rename = "relationship-updated")]
    RelationshipUpdated {
        uid: String,
        status: String,
        timestamp: u64,
    },
    #[serde(rename = "new-feed-post")]
    NewFeedPost {
        id: String,
        body: serde_json::Value,
        cursor: String,
        #[serde(rename = "createdAt")]
        created_at: u64,
    },
    #[serde(rename = "kv-batch-update")]
    KvBatchUpdate { changes: Vec<KvBatchChange> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LateVersionedValue {
    pub value: String,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KvBatchChange {
    pub key: String,
    pub value: Option<String>,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DurableUpdateBody {
    Core(CoreUpdateBody),
    Late(LateDurableUpdate),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableUpdateContainer {
    pub id: String,
    pub seq: u64,
    pub body: DurableUpdateBody,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EphemeralUpdate {
    #[serde(rename = "activity")]
    Activity {
        id: String,
        active: bool,
        #[serde(rename = "activeAt")]
        active_at: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        thinking: Option<bool>,
    },
    #[serde(rename = "machine-activity")]
    MachineActivity {
        id: String,
        active: bool,
        #[serde(rename = "activeAt")]
        active_at: u64,
    },
    #[serde(rename = "machine-status")]
    MachineStatus {
        #[serde(rename = "machineId")]
        machine_id: String,
        online: bool,
        timestamp: u64,
    },
    #[serde(rename = "usage")]
    Usage {
        id: String,
        key: String,
        tokens: std::collections::BTreeMap<String, u64>,
        cost: std::collections::BTreeMap<String, f64>,
        timestamp: u64,
    },
}
