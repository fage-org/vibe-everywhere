//! F1: Device registration & pairing

use std::sync::Arc;

use crate::flows::FlowResult;
use crate::test_context::TestContext;

pub async fn run(ctx: Arc<TestContext>) -> FlowResult {
    let start = std::time::Instant::now();

    let result = run_impl(&ctx).await;

    match result {
        Ok(()) => FlowResult::pass("f1", start.elapsed().as_secs_f64()),
        Err(e) => FlowResult::fail("f1", &e.to_string()),
    }
}

async fn run_impl(ctx: &TestContext) -> anyhow::Result<()> {
    let client = &ctx.client;

    // Integration setup completed pairing via the REAL /api/auth/register-device
    // and /api/auth/pair endpoints (ECC HIGH-03 fix). Verify the state is correct.

    // Step 1: Verify host_id exists
    let host_id = ctx
        .host_id
        .ok_or_else(|| anyhow::anyhow!("F1 requires host_id (integration mode not set up)"))?;

    // Step 2: List hosts — verify our host is paired
    let hosts = client
        .list_hosts()
        .await
        .map_err(|e| anyhow::anyhow!("list_hosts: {e}"))?;

    if hosts.hosts.is_empty() {
        anyhow::bail!("Expected at least one paired host, got none");
    }

    let our_host = hosts.hosts.iter().find(|h| h.host_id == host_id);

    let our_host =
        our_host.ok_or_else(|| anyhow::anyhow!("Our host_id {host_id} not in host list"))?;

    if our_host.pair_status != ve_shared::types::PairStatus::Paired {
        anyhow::bail!(
            "Expected pair_status='paired', got '{:?}'",
            our_host.pair_status
        );
    }

    tracing::info!("Host {host_id} pair_status verified: paired");

    // Step 3: Verify device_id was registered
    let _device_id = ctx
        .device_id
        .ok_or_else(|| anyhow::anyhow!("F1 requires device_id (integration mode not set up)"))?;

    // Step 4: Verify pair_code is consumed (should be empty/null after pairing)
    // After pairing, pair_code and qr_payload should be cleared
    if our_host.pair_code.as_ref().is_some_and(|s| !s.is_empty()) {
        anyhow::bail!(
            "pair_code should be empty after pairing, got: {:?}",
            our_host.pair_code
        );
    }

    if our_host.qr_payload.as_ref().is_some_and(|s| !s.is_empty()) {
        anyhow::bail!(
            "qr_payload should be empty after pairing, got: {:?}",
            our_host.qr_payload
        );
    }

    tracing::info!("Pairing credentials consumed correctly");

    // Step 5: Error path — pairing-status with invalid host_id
    let bad_host_id = uuid::Uuid::new_v4();
    let result = client.pairing_status(bad_host_id, "invalid-secret").await;
    if result.is_ok() {
        anyhow::bail!("Expected error for invalid host_id pairing-status, but got OK");
    }

    tracing::info!("Error path verified: invalid host_id rejected");

    // Step 6: Verify the client token works (implicit from all above calls succeeding)
    tracing::info!("Client token authenticated successfully across all endpoints");

    Ok(())
}
