#![forbid(unsafe_code)]

//! Canonical shared protocol crate for the Happy-aligned rebuild.

pub mod compat;
mod serde_helpers;

pub mod legacy_protocol;
pub mod message_meta;
pub mod messages;
pub mod session_protocol;
pub mod voice;

pub use compat::{
    CompatibilityVectorFile, JsonFixture, compatibility_vector_files,
    invalid_session_envelope_fixtures, legacy_message_fixtures, message_content_fixtures,
    message_meta_fixtures, session_envelope_fixtures, update_container_fixtures,
    voice_response_fixtures,
};
pub use legacy_protocol::{
    AgentMessage, AgentMessageContent, AgentMessageContentError, LegacyMessageContent, UserMessage,
    UserMessageContent,
};
pub use message_meta::MessageMeta;
pub use messages::{
    ApiMessage, ApiUpdateMachineState, ApiUpdateNewMessage, ApiUpdateSessionState, CoreUpdateBody,
    CoreUpdateContainer, MessageContent, ResumeSessionRpcParams, SessionMessage,
    SessionMessageContent, SessionProtocolMessage, SpawnSessionRpcParams, SpawnSessionRpcResult,
    StopDaemonRpcResult, Update, UpdateBody, UpdateMachineBody, UpdateNewMessageBody,
    UpdateSessionBody, VersionedEncryptedValue, VersionedMachineEncryptedValue,
    VersionedNullableEncryptedValue,
};
pub use session_protocol::{
    CreateEnvelopeOptions, SessionEnvelope, SessionEnvelopeValidationError, SessionEvent,
    SessionFileEvent, SessionFileImage, SessionRole, SessionServiceEvent, SessionStartEvent,
    SessionStopEvent, SessionTextEvent, SessionToolCallEndEvent, SessionToolCallStartEvent,
    SessionTurnEndEvent, SessionTurnEndStatus, SessionTurnStartEvent, create_envelope,
};
pub use voice::{VoiceTokenAllowed, VoiceTokenDenied, VoiceTokenDeniedReason, VoiceTokenResponse};
