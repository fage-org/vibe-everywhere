use std::collections::BTreeMap;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use vibe_wire::SessionMessageContent;

use crate::storage::db::{
    AccessKeyRecord, ArtifactRecord, FeedPostRecord, KvRecord, MachineRecord, PushTokenRecord,
    SessionMessageRecord, SessionRecord,
};

#[derive(Debug, Clone)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    error: String,
}

impl ApiError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrLoadSessionBody {
    pub tag: String,
    pub metadata: String,
    pub agent_state: Option<String>,
    pub data_encryption_key: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionHttpRecord {
    pub id: String,
    pub seq: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub active: bool,
    pub active_at: u64,
    pub metadata: String,
    pub metadata_version: u64,
    pub agent_state: Option<String>,
    pub agent_state_version: u64,
    pub data_encryption_key: Option<String>,
    #[serde(rename = "lastMessage", skip_serializing_if = "Option::is_none")]
    pub last_message: Option<()>,
}

impl SessionHttpRecord {
    pub fn from_record(record: &SessionRecord, include_last_message: bool) -> Self {
        Self {
            id: record.id.clone(),
            seq: record.seq,
            created_at: record.created_at,
            updated_at: record.updated_at,
            active: record.active,
            active_at: record.active_at,
            metadata: record.metadata.clone(),
            metadata_version: record.metadata_version,
            agent_state: record.agent_state.clone(),
            agent_state_version: record.agent_state_version,
            data_encryption_key: record.data_encryption_key.clone(),
            last_message: include_last_message.then_some(()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionHttpRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActiveSessionListResponse {
    pub sessions: Vec<SessionHttpRecord>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorPagedSessionListResponse {
    pub sessions: Vec<SessionHttpRecord>,
    pub next_cursor: Option<String>,
    pub has_next: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateOrLoadSessionResponse {
    pub session: SessionHttpRecord,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionHistoryResponse {
    pub messages: Vec<SessionMessageHttpRecord>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ActiveSessionsQuery {
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CursorPagedSessionsQuery {
    pub cursor: Option<String>,
    pub limit: Option<usize>,
    pub changed_since: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionPath {
    pub session_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct V3MessagesQuery {
    pub after_seq: Option<u64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkMessageInput {
    pub content: String,
    pub local_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMessageHttpRecord {
    pub id: String,
    pub seq: u64,
    pub local_id: Option<String>,
    pub content: SessionMessageContent,
    pub created_at: u64,
    pub updated_at: u64,
}

impl SessionMessageHttpRecord {
    pub fn from_record(record: &SessionMessageRecord) -> Self {
        Self {
            id: record.id.clone(),
            seq: record.seq,
            local_id: record.local_id.clone(),
            content: record.content.clone(),
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct V3SendMessagesBody {
    pub messages: Vec<BulkMessageInput>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMessageSendRecord {
    pub id: String,
    pub seq: u64,
    pub local_id: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl SessionMessageSendRecord {
    pub fn from_record(record: &SessionMessageRecord) -> Self {
        Self {
            id: record.id.clone(),
            seq: record.seq,
            local_id: record.local_id.clone(),
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct V3MessagesResponse {
    pub messages: Vec<SessionMessageHttpRecord>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct V3SendMessagesResponse {
    pub messages: Vec<SessionMessageSendRecord>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrLoadMachineBody {
    pub id: String,
    pub metadata: String,
    pub daemon_state: Option<String>,
    pub data_encryption_key: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MachineHttpRecord {
    pub id: String,
    pub seq: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub active: bool,
    pub active_at: u64,
    pub metadata: String,
    pub metadata_version: u64,
    pub daemon_state: Option<String>,
    pub daemon_state_version: u64,
    pub data_encryption_key: Option<String>,
}

impl MachineHttpRecord {
    pub fn from_record(record: &MachineRecord) -> Self {
        Self {
            id: record.id.clone(),
            seq: record.seq,
            created_at: record.created_at,
            updated_at: record.updated_at,
            active: record.active,
            active_at: record.active_at,
            metadata: record.metadata.clone(),
            metadata_version: record.metadata_version,
            daemon_state: record.daemon_state.clone(),
            daemon_state_version: record.daemon_state_version,
            data_encryption_key: record.data_encryption_key.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrLoadMachineHttpRecord {
    pub id: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub active: bool,
    pub active_at: u64,
    pub metadata: String,
    pub metadata_version: u64,
    pub daemon_state: Option<String>,
    pub daemon_state_version: u64,
    pub data_encryption_key: Option<String>,
}

impl CreateOrLoadMachineHttpRecord {
    pub fn from_record(record: &MachineRecord) -> Self {
        Self {
            id: record.id.clone(),
            created_at: record.created_at,
            updated_at: record.updated_at,
            active: record.active,
            active_at: record.active_at,
            metadata: record.metadata.clone(),
            metadata_version: record.metadata_version,
            daemon_state: record.daemon_state.clone(),
            daemon_state_version: record.daemon_state_version,
            data_encryption_key: record.data_encryption_key.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateOrLoadMachineResponse {
    pub machine: CreateOrLoadMachineHttpRecord,
}

#[derive(Debug, Clone, Serialize)]
pub struct MachineDetailResponse {
    pub machine: MachineHttpRecord,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MachinePath {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionCheckBody {
    pub platform: String,
    pub version: String,
    pub app_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountProfileResponse {
    pub id: String,
    pub timestamp: u64,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub avatar: Option<Value>,
    pub github: Option<Value>,
    pub connected_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSettingsResponse {
    pub settings: Option<String>,
    pub settings_version: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountSettingsBody {
    pub settings: Option<String>,
    pub expected_version: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountSettingsSuccess {
    pub success: bool,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountSettingsConflict {
    pub success: bool,
    pub error: String,
    pub current_version: u64,
    pub current_settings: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageQueryBody {
    pub session_id: Option<String>,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub group_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UsageBucket {
    pub timestamp: u64,
    pub tokens: BTreeMap<String, u64>,
    pub cost: BTreeMap<String, f64>,
    pub report_count: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageQueryResponse {
    pub usage: Vec<UsageBucket>,
    pub group_by: String,
    pub total_reports: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KvPath {
    pub key: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct KvEntry {
    pub key: String,
    pub value: Option<String>,
    pub version: u64,
}

impl KvEntry {
    pub fn from_record(record: &KvRecord) -> Self {
        Self {
            key: record.key.clone(),
            value: record.value.clone(),
            version: record.version,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct KvListResponse {
    pub items: Vec<KvEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KvListQuery {
    pub prefix: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KvBulkGetBody {
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KvBulkGetResponse {
    pub values: Vec<KvEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KvMutationInput {
    pub key: String,
    pub value: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KvMutateBody {
    pub mutations: Vec<KvMutationInput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KvMutateResult {
    pub key: String,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct KvMutateConflict {
    pub key: String,
    pub error: String,
    pub version: u64,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KvMutateSuccessResponse {
    pub success: bool,
    pub results: Vec<KvMutateResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KvMutateConflictResponse {
    pub success: bool,
    pub errors: Vec<KvMutateConflict>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PushTokenBody {
    pub token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PushTokenItem {
    pub id: String,
    pub token: String,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    #[serde(rename = "updatedAt")]
    pub updated_at: u64,
}

impl PushTokenItem {
    pub fn from_record(record: &PushTokenRecord) -> Self {
        Self {
            id: record.id.clone(),
            token: record.token.clone(),
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PushTokenListResponse {
    pub tokens: Vec<PushTokenItem>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceTokenBody {
    pub agent_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArtifactPath {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactInfo {
    pub id: String,
    pub header: String,
    pub header_version: u64,
    pub data_encryption_key: String,
    pub seq: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    pub id: String,
    pub header: String,
    pub header_version: u64,
    pub body: String,
    pub body_version: u64,
    pub data_encryption_key: String,
    pub seq: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl ArtifactInfo {
    pub fn from_record(record: &ArtifactRecord) -> Self {
        Self {
            id: record.id.clone(),
            header: record.header.clone(),
            header_version: record.header_version,
            data_encryption_key: record.data_encryption_key.clone(),
            seq: record.seq,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

impl Artifact {
    pub fn from_record(record: &ArtifactRecord) -> Self {
        Self {
            id: record.id.clone(),
            header: record.header.clone(),
            header_version: record.header_version,
            body: record.body.clone(),
            body_version: record.body_version,
            data_encryption_key: record.data_encryption_key.clone(),
            seq: record.seq,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateArtifactBody {
    pub id: String,
    pub header: String,
    pub body: String,
    pub data_encryption_key: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArtifactBody {
    pub header: Option<String>,
    pub expected_header_version: Option<u64>,
    pub body: Option<String>,
    pub expected_body_version: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactVersionField {
    pub value: String,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArtifactResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_version: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_version: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_header_version: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_header: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_body_version: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_body: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessKeyPath {
    pub session_id: String,
    pub machine_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePushTokenPath {
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VendorPath {
    pub vendor: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserPath {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserSearchQuery {
    pub query: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FriendMutationBody {
    pub uid: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub id: String,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: String,
    pub avatar: Option<Value>,
    pub bio: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserProfileResponse {
    pub user: Option<UserProfile>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserSearchResponse {
    pub users: Vec<UserProfile>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FriendsListResponse {
    pub friends: Vec<UserProfile>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedPostResponse {
    pub id: String,
    pub body: Value,
    pub repeat_key: Option<String>,
    pub cursor: String,
    pub created_at: u64,
}

impl FeedPostResponse {
    pub fn from_record(record: &FeedPostRecord) -> Self {
        Self {
            id: record.id.clone(),
            body: record.body.clone(),
            repeat_key: record.repeat_key.clone(),
            cursor: record.cursor.clone(),
            created_at: record.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedListResponse {
    pub items: Vec<FeedPostResponse>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessKeyValue {
    pub data: String,
    pub data_version: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl AccessKeyValue {
    pub fn from_record(record: &AccessKeyRecord) -> Self {
        Self {
            data: record.data.clone(),
            data_version: record.data_version,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccessKeyResponse {
    pub access_key: Option<AccessKeyValue>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAccessKeyBody {
    pub data: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccessKeyBody {
    pub data: String,
    pub expected_version: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccessKeyResponse {
    pub success: bool,
    pub access_key: AccessKeyValue,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccessKeyResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_version: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_data: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeedQuery {
    pub before: Option<String>,
    pub after: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageReportPayload {
    pub key: String,
    pub session_id: Option<String>,
    pub tokens: BTreeMap<String, u64>,
    pub cost: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessKeyGetPayload {
    pub session_id: String,
    pub machine_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactReadPayload {
    pub artifact_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactCreatePayload {
    pub id: String,
    pub header: String,
    pub body: String,
    pub data_encryption_key: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactSocketVersionPayload {
    pub data: String,
    pub expected_version: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactUpdatePayload {
    pub artifact_id: String,
    pub header: Option<ArtifactSocketVersionPayload>,
    pub body: Option<ArtifactSocketVersionPayload>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeAuthBody {
    pub public_key: String,
    pub challenge: String,
    pub signature: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthRequestBody {
    pub public_key: String,
    pub supports_v2: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountAuthRequestBody {
    pub public_key: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponseBody {
    pub public_key: String,
    pub response: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatusQuery {
    pub public_key: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocketMessagePayload {
    pub sid: String,
    pub message: String,
    pub local_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocketUpdateMetadataPayload {
    pub sid: String,
    pub metadata: String,
    pub expected_version: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocketUpdateStatePayload {
    pub sid: String,
    #[serde(default, deserialize_with = "deserialize_present_nullable_string")]
    pub agent_state: Option<Option<String>>,
    pub expected_version: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionAlivePayload {
    pub sid: String,
    pub time: u64,
    pub thinking: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionEndPayload {
    pub sid: String,
    pub time: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MachineAlivePayload {
    pub machine_id: String,
    pub time: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MachineUpdateMetadataPayload {
    pub machine_id: String,
    pub metadata: String,
    pub expected_version: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MachineUpdateStatePayload {
    pub machine_id: String,
    pub daemon_state: String,
    pub expected_version: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcRegisterPayload {
    pub method: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcCallPayload {
    pub method: String,
    pub params: serde_json::Value,
}

fn deserialize_present_nullable_string<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(Some)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        ChallengeAuthBody, RpcCallPayload, SocketUpdateMetadataPayload, SocketUpdateStatePayload,
    };

    #[test]
    fn challenge_auth_body_accepts_happy_camel_case_fields() {
        let body: ChallengeAuthBody = serde_json::from_value(json!({
            "publicKey": "pk",
            "challenge": "challenge",
            "signature": "sig",
        }))
        .unwrap();

        assert_eq!(body.public_key, "pk");
    }

    #[test]
    fn socket_update_metadata_payload_accepts_expected_version() {
        let payload: SocketUpdateMetadataPayload = serde_json::from_value(json!({
            "sid": "session-1",
            "metadata": "ciphertext",
            "expectedVersion": 3,
        }))
        .unwrap();

        assert_eq!(payload.expected_version, 3);
    }

    #[test]
    fn rpc_call_payload_accepts_structured_params() {
        let payload: RpcCallPayload = serde_json::from_value(json!({
            "method": "machine:call",
            "params": { "nested": true },
        }))
        .unwrap();

        assert_eq!(payload.params, json!({ "nested": true }));
    }

    #[test]
    fn socket_update_state_payload_distinguishes_missing_and_null_agent_state() {
        let missing: SocketUpdateStatePayload = serde_json::from_value(json!({
            "sid": "session-1",
            "expectedVersion": 3,
        }))
        .unwrap();
        assert_eq!(missing.agent_state, None);

        let null_value: SocketUpdateStatePayload = serde_json::from_value(json!({
            "sid": "session-1",
            "agentState": null,
            "expectedVersion": 3,
        }))
        .unwrap();
        assert_eq!(null_value.agent_state, Some(None));

        let string_value: SocketUpdateStatePayload = serde_json::from_value(json!({
            "sid": "session-1",
            "agentState": "ready",
            "expectedVersion": 3,
        }))
        .unwrap();
        assert_eq!(string_value.agent_state, Some(Some("ready".into())));
    }
}
