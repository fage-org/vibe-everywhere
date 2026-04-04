use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    match vibe_cli::bootstrap::run_cli().await {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}
