#[tokio::main]
async fn main() -> Result<(), vibe_app_logs::AppLogsError> {
    vibe_app_logs::run().await
}
