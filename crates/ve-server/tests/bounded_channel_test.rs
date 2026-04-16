//! Tests for bounded WebSocket channels
//!
//! Tests for bounded channel behavior in WebSocket handlers.

use tokio::sync::mpsc;
use ve_shared::proto::WsEnvelope;

/// Default bounded channel capacity for WebSocket connections
const WS_CHANNEL_CAPACITY: usize = 256;

/// Create a bounded channel for WebSocket messages
fn create_ws_channel() -> (mpsc::Sender<WsEnvelope>, mpsc::Receiver<WsEnvelope>) {
    mpsc::channel(WS_CHANNEL_CAPACITY)
}

/// Create a bounded channel with custom capacity
fn create_ws_channel_with_capacity(
    capacity: usize,
) -> (mpsc::Sender<WsEnvelope>, mpsc::Receiver<WsEnvelope>) {
    mpsc::channel(capacity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn bounded_channel_created_with_default_capacity() {
        let (tx, mut rx) = create_ws_channel();

        // Should be able to send without waiting (under capacity)
        let msg = WsEnvelope::new("test", serde_json::json!({}));
        let send_result = tx.try_send(msg);
        assert!(send_result.is_ok());

        // Should receive the message
        let received = rx.recv().await;
        assert!(received.is_some());
    }

    #[tokio::test]
    async fn bounded_channel_respects_capacity() {
        let capacity: usize = 5;
        let (tx, _rx) = create_ws_channel_with_capacity(capacity);

        // Fill the channel
        for i in 0..capacity {
            let msg = WsEnvelope::new("test", serde_json::json!({ "seq": i }));
            let result = tx.try_send(msg);
            assert!(result.is_ok(), "Send {} should succeed", i);
        }

        // Next send should fail (channel full)
        let msg = WsEnvelope::new("test", serde_json::json!({ "seq": capacity }));
        let result = tx.try_send(msg);
        assert!(result.is_err());

        // Verify it's a full error
        if let Err(e) = result {
            assert!(matches!(e, mpsc::error::TrySendError::Full(_)));
        }
    }

    #[tokio::test]
    async fn bounded_channel_allows_send_when_space_available() {
        let (tx, mut rx) = create_ws_channel();

        // Send a message
        let msg = WsEnvelope::new("test", serde_json::json!({ "data": "hello" }));
        tx.send(msg).await.unwrap();

        // Receive it
        let received = rx.recv().await.unwrap();
        assert_eq!(received.r#type, "test");

        // Now there's space again
        let msg2 = WsEnvelope::new("test2", serde_json::json!({}));
        let result = tx.try_send(msg2);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn bounded_channel_sender_cloned_shares_capacity() {
        let capacity: usize = 3;
        let (tx, _rx) = create_ws_channel_with_capacity(capacity);

        let tx2 = tx.clone();

        // Fill using first sender
        for i in 0..capacity {
            let msg = WsEnvelope::new("test", serde_json::json!({ "from": "tx1", "seq": i }));
            let result = tx.try_send(msg);
            assert!(result.is_ok());
        }

        // Second sender should also see full channel
        let msg = WsEnvelope::new("test", serde_json::json!({ "from": "tx2" }));
        let result = tx2.try_send(msg);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn bounded_channel_receiver_drop_closes_channel() {
        let (tx, rx) = create_ws_channel();

        // Drop receiver
        drop(rx);

        // Sender should detect closed channel
        let msg = WsEnvelope::new("test", serde_json::json!({}));
        let result = tx.send(msg).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn bounded_channel_default_capacity_is_256() {
        let (tx, _rx) = create_ws_channel();

        // Fill up to capacity
        let mut sent_count = 0;
        for i in 0..300 {
            let msg = WsEnvelope::new("test", serde_json::json!({ "seq": i }));
            match tx.try_send(msg) {
                Ok(_) => sent_count += 1,
                Err(mpsc::error::TrySendError::Full(_)) => break,
                Err(mpsc::error::TrySendError::Closed(_)) => break,
            }
        }

        // Should have sent exactly 256 messages before hitting capacity
        assert_eq!(sent_count, WS_CHANNEL_CAPACITY);
    }

    #[tokio::test]
    async fn bounded_channel_drain_and_refill() {
        let (tx, mut rx) = create_ws_channel();

        // Fill the channel
        for i in 0..WS_CHANNEL_CAPACITY {
            let msg = WsEnvelope::new("test", serde_json::json!({ "seq": i }));
            tx.send(msg).await.unwrap();
        }

        // Drain all messages
        let mut count = 0;
        while let Ok(msg) = rx.try_recv() {
            count += 1;
            assert_eq!(msg.r#type, "test");
        }
        assert_eq!(count, WS_CHANNEL_CAPACITY);

        // Should be able to fill again
        for i in 0..WS_CHANNEL_CAPACITY {
            let msg = WsEnvelope::new("test", serde_json::json!({ "seq": i }));
            let result = tx.try_send(msg);
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn bounded_channel_sender_drop_notifies_receiver() {
        let (tx, mut rx) = create_ws_channel();

        // Send one message
        let msg = WsEnvelope::new("test", serde_json::json!({}));
        tx.send(msg).await.unwrap();

        // Drop sender
        drop(tx);

        // Receive the message
        let received = rx.recv().await;
        assert!(received.is_some());

        // Next recv should return None (channel closed)
        let next = rx.recv().await;
        assert!(next.is_none());
    }

    #[tokio::test]
    async fn bounded_channel_preserves_message_order() {
        let (tx, mut rx) = create_ws_channel();

        // Send multiple messages
        for i in 0..10 {
            let msg = WsEnvelope::new("test", serde_json::json!({ "seq": i }));
            tx.send(msg).await.unwrap();
        }

        // Receive and verify order
        for expected in 0..10 {
            let received = rx.recv().await.unwrap();
            let seq = received.payload.get("seq").unwrap().as_u64().unwrap();
            assert_eq!(seq, expected);
        }
    }
}
