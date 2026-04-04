#[tokio::main]
async fn main() -> Result<(), vibe_server::ServerError> {
    vibe_server::run().await
}
