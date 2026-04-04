use anyhow::{Result, anyhow};
use tokio::{sync::mpsc, task::JoinHandle};
use vibe_agent::{
    api::DecryptedSession,
    encryption::{EncryptionVariant, encode_base64, encrypt_json},
};
use vibe_wire::SessionTurnEndStatus;

use crate::{
    agent::adapters::NormalizedEvent,
    api::{CliApiClient, QueuedMessage},
    session_protocol_mapper::{
        map_agent_output, map_normalized_event, map_turn_end, map_turn_start, map_turn_text,
        map_user_prompt,
    },
};

#[derive(Debug)]
enum TransportEvent {
    Payload {
        payload: serde_json::Value,
        local_id: String,
    },
}

#[derive(Debug, Clone, Copy)]
struct SessionTransportContext {
    encryption_key: [u8; 32],
    encryption_variant: EncryptionVariant,
}

pub struct DefaultTransport {
    sender: mpsc::Sender<TransportEvent>,
    worker: JoinHandle<Result<()>>,
}

impl DefaultTransport {
    pub fn new(api: &CliApiClient, session: &DecryptedSession) -> Self {
        let api = api.clone();
        let session_id = session.id.clone();
        let context = SessionTransportContext {
            encryption_key: session.encryption.key,
            encryption_variant: session.encryption.variant,
        };
        let (sender, mut receiver) = mpsc::channel(64);
        let worker = tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match event {
                    TransportEvent::Payload { payload, local_id } => {
                        let content = encode_base64(&encrypt_json(
                            &context.encryption_key,
                            context.encryption_variant,
                            &payload,
                        )?);
                        api.post_messages_v3(
                            &session_id,
                            vec![QueuedMessage { content, local_id }],
                        )
                        .await?;
                    }
                }
            }
            Ok(())
        });

        Self { sender, worker }
    }

    pub async fn send_user_prompt(&self, prompt: &str) -> Result<()> {
        self.enqueue_payload(
            map_user_prompt(prompt),
            format!("user-{}", uuid::Uuid::now_v7()),
        )
        .await
    }

    pub async fn start_agent_turn(&self) -> Result<String> {
        let turn_id = format!("turn-{}", uuid::Uuid::now_v7());
        self.enqueue_payload(
            map_turn_start(&turn_id),
            format!("agent-turn-start-{}", uuid::Uuid::now_v7()),
        )
        .await?;
        Ok(turn_id)
    }

    pub async fn send_agent_text(&self, turn_id: &str, text: &str) -> Result<()> {
        if text.trim().is_empty() {
            return Ok(());
        }
        self.enqueue_payload(
            map_turn_text(turn_id, text),
            format!("agent-turn-text-{}", uuid::Uuid::now_v7()),
        )
        .await
    }

    pub async fn send_agent_event(&self, turn_id: &str, event: &NormalizedEvent) -> Result<()> {
        let Some(payload) = map_normalized_event(turn_id, event) else {
            return Ok(());
        };
        self.enqueue_payload(payload, format!("agent-event-{}", uuid::Uuid::now_v7()))
            .await
    }

    pub async fn finish_agent_turn(
        &self,
        turn_id: &str,
        full_output: &str,
        status: SessionTurnEndStatus,
    ) -> Result<()> {
        self.enqueue_payload(
            map_turn_end(turn_id, status),
            format!("agent-turn-end-{}", uuid::Uuid::now_v7()),
        )
        .await?;
        if !full_output.trim().is_empty() {
            self.enqueue_payload(
                map_agent_output(full_output),
                format!("agent-legacy-{}", uuid::Uuid::now_v7()),
            )
            .await?;
        }
        Ok(())
    }

    async fn enqueue_payload(&self, payload: serde_json::Value, local_id: String) -> Result<()> {
        self.sender
            .send(TransportEvent::Payload { payload, local_id })
            .await
            .map_err(|_| anyhow!("transport worker is not running"))?;
        Ok(())
    }

    pub async fn close(self) -> Result<()> {
        drop(self.sender);
        self.worker
            .await
            .map_err(|error| anyhow!("transport worker join failed: {error}"))??;
        Ok(())
    }
}
