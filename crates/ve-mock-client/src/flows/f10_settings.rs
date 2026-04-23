//! F10: Settings — get/update notification preferences

use std::sync::Arc;

use crate::fixtures;
use crate::flows::FlowResult;
use crate::test_context::TestContext;

pub async fn run(ctx: Arc<TestContext>) -> FlowResult {
    let start = std::time::Instant::now();

    let result = run_impl(&ctx).await;

    match result {
        Ok(()) => FlowResult::pass("f10", start.elapsed().as_secs_f64()),
        Err(e) => FlowResult::fail("f10", &e.to_string()),
    }
}

async fn run_impl(ctx: &TestContext) -> anyhow::Result<()> {
    let client = &ctx.client;

    // Step 1: Get current notification preferences (defaults via COALESCE)
    let prefs = client
        .get_notification_preferences()
        .await
        .map_err(|e| anyhow::anyhow!("get notification preferences: {e}"))?;

    // Verify defaults exist (COALESCE should return defaults for missing rows)
    tracing::info!(
        "Default prefs: enabled={}, permission={}, task_completed={}, task_failed={}, session_error={}",
        prefs.enabled,
        prefs.permission_request_enabled,
        prefs.task_completed_enabled,
        prefs.task_failed_enabled,
        prefs.session_error_enabled
    );

    // Step 2: Update notification preferences
    client
        .update_notification_preferences(true, true, true, true, true)
        .await
        .map_err(|e| anyhow::anyhow!("update notification preferences: {e}"))?;

    // Step 3: Get again and verify changes persisted
    let reloaded = client
        .get_notification_preferences()
        .await
        .map_err(|e| anyhow::anyhow!("get after update: {e}"))?;

    if !reloaded.enabled {
        anyhow::bail!("Prefs not persisted: enabled=false (expected true)");
    }

    tracing::info!(
        "Settings prefs updated and verified: enabled={}",
        reloaded.enabled
    );

    // Step 4: Error path — update with invalid device (should fail gracefully)
    let bad_client = crate::client::MockClient::new(
        ctx.server_url.clone(),
        fixtures::fake_token_for_nonexistent_device(),
    );

    let result = bad_client.get_notification_preferences().await;
    if result.is_ok() {
        anyhow::bail!("Expected 401/403 for invalid token, but got OK");
    }

    tracing::info!("Error path verified: invalid token rejected");

    Ok(())
}
