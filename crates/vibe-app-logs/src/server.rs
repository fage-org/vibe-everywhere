use std::{
    fs::{File, OpenOptions, create_dir_all},
    io::Write,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use chrono::{DateTime, Local};
use serde::Deserialize;
use tokio::net::TcpListener;

use crate::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("failed to prepare logs directory: {0}")]
    CreateLogsDir(std::io::Error),
    #[error("failed to open the log file: {0}")]
    OpenLogFile(std::io::Error),
    #[error("failed to start log server: {0}")]
    Startup(String),
}

#[derive(Debug, Clone)]
pub struct LogRuntime {
    state: AppState,
    pub log_path: PathBuf,
}

#[derive(Debug, Clone)]
struct AppState {
    sink: Arc<Mutex<File>>,
}

#[derive(Debug, Deserialize)]
struct LogPayload {
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    level: Option<String>,
    message: String,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    platform: Option<String>,
}

impl LogRuntime {
    pub fn new(config: &Config) -> Result<Self, ServerError> {
        create_dir_all(&config.logs_dir).map_err(ServerError::CreateLogsDir)?;

        let now = Local::now();
        let filename = now.format("%Y-%m-%d-%H-%M-%S.log").to_string();
        let log_path = config.logs_dir.join(filename);
        let sink = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(ServerError::OpenLogFile)?;

        Ok(Self {
            state: AppState {
                sink: Arc::new(Mutex::new(sink)),
            },
            log_path,
        })
    }

    pub fn router(&self) -> Router {
        Router::new()
            .route("/logs", post(ingest_logs).options(preflight))
            .with_state(self.state.clone())
    }
}

pub async fn run(config: Config) -> Result<(), ServerError> {
    let runtime = LogRuntime::new(&config)?;
    let bind_addr = config.bind_addr();
    let listener = TcpListener::bind(bind_addr)
        .await
        .map_err(|error| ServerError::Startup(error.to_string()))?;

    print_startup_banner(bind_addr, &runtime.log_path);

    axum::serve(listener, runtime.router())
        .await
        .map_err(|error| ServerError::Startup(error.to_string()))?;
    Ok(())
}

fn print_startup_banner(bind_addr: SocketAddr, log_path: &PathBuf) {
    println!("App log receiver listening on http://{bind_addr}");
    println!("Writing to {}", log_path.display());
}

async fn preflight() -> Response {
    with_cors(StatusCode::NO_CONTENT.into_response())
}

async fn ingest_logs(State(state): State<AppState>, body: Bytes) -> Response {
    let payload: LogPayload = match serde_json::from_slice(&body) {
        Ok(payload) => payload,
        Err(_) => {
            return with_cors(
                (
                    StatusCode::BAD_REQUEST,
                    axum::Json(serde_json::json!({ "error": "bad request" })),
                )
                    .into_response(),
            );
        }
    };

    let line = build_log_line(&payload);
    match write_line(&state, &line) {
        Ok(()) => with_cors(
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "ok": true })),
            )
                .into_response(),
        ),
        Err(_) => with_cors(
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": "write failed" })),
            )
                .into_response(),
        ),
    }
}

fn build_log_line(payload: &LogPayload) -> String {
    let time = payload
        .timestamp
        .as_deref()
        .map(format_time)
        .unwrap_or_else(|| Local::now().format("%H:%M:%S%.3f").to_string());
    let level = payload
        .level
        .as_deref()
        .unwrap_or("info")
        .to_ascii_uppercase();
    let source = payload.source.as_deref().unwrap_or("app");
    let platform = payload.platform.as_deref().unwrap_or("?");

    format!(
        "[{time}] [{level:<5}] [{source}/{platform}] {}\n",
        payload.message
    )
}

fn format_time(raw: &str) -> String {
    DateTime::parse_from_rfc3339(raw)
        .map(|value| {
            value
                .with_timezone(&Local)
                .format("%H:%M:%S%.3f")
                .to_string()
        })
        .unwrap_or_else(|_| raw.to_string())
}

fn write_line(state: &AppState, line: &str) -> Result<(), std::io::Error> {
    {
        let mut sink = state.sink.lock().expect("log sink mutex poisoned");
        sink.write_all(line.as_bytes())?;
        sink.flush()?;
    }
    print!("{line}");
    Ok(())
}

fn with_cors(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("POST, OPTIONS"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("Content-Type"),
    );
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    response
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use reqwest::{Client, Method};
    use tempfile::TempDir;
    use tokio::{net::TcpListener, task::JoinHandle};

    use crate::config::Config;

    use super::{LogRuntime, ServerError, build_log_line};

    async fn spawn_runtime() -> (TempDir, String, std::path::PathBuf, JoinHandle<()>) {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::from_test_sources(
            "127.0.0.1",
            Some(0),
            None,
            Some(temp_dir.path().as_os_str().to_owned()),
        )
        .unwrap();
        let runtime = LogRuntime::new(&config).unwrap();
        let log_path = runtime.log_path.clone();
        let listener = TcpListener::bind(config.bind_addr()).await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = runtime.router();
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (temp_dir, format!("http://{addr}"), log_path, handle)
    }

    #[test]
    fn builds_happy_compatible_log_line() {
        let line = build_log_line(&super::LogPayload {
            timestamp: Some("2025-01-02T03:04:05.678Z".into()),
            level: Some("warn".into()),
            message: "hello".into(),
            source: Some("mobile".into()),
            platform: Some("ios".into()),
        });
        assert!(line.contains("[WARN "));
        assert!(line.contains("[mobile/ios] hello"));
    }

    #[tokio::test]
    async fn startup_and_ingestion_smoke_test() {
        let (_temp_dir, base_url, log_path, server) = spawn_runtime().await;
        let client = Client::new();

        let response = client
            .post(format!("{base_url}/logs"))
            .json(&serde_json::json!({
                "timestamp": "2025-01-02T03:04:05.678Z",
                "level": "info",
                "message": "hello from app",
                "source": "mobile",
                "platform": "ios",
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let log_contents = std::fs::read_to_string(&log_path).unwrap();
        assert!(log_contents.contains("hello from app"));

        server.abort();
    }

    #[tokio::test]
    async fn invalid_payload_returns_bad_request() {
        let (_temp_dir, base_url, _log_path, server) = spawn_runtime().await;
        let client = Client::new();

        let response = client
            .post(format!("{base_url}/logs"))
            .body("not-json")
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        server.abort();
    }

    #[tokio::test]
    async fn preflight_returns_cors_headers() {
        let (_temp_dir, base_url, _log_path, server) = spawn_runtime().await;
        let client = Client::new();

        let response = client
            .request(Method::OPTIONS, format!("{base_url}/logs"))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            response
                .headers()
                .get("access-control-allow-origin")
                .and_then(|value| value.to_str().ok()),
            Some("*")
        );
        assert_eq!(
            response
                .headers()
                .get("access-control-allow-methods")
                .and_then(|value| value.to_str().ok()),
            Some("POST, OPTIONS")
        );
        assert_eq!(
            response
                .headers()
                .get("access-control-allow-headers")
                .and_then(|value| value.to_str().ok()),
            Some("Content-Type")
        );

        server.abort();
    }

    #[test]
    fn runtime_creation_fails_when_logs_path_is_not_a_directory() {
        let temp_dir = TempDir::new().unwrap();
        let home_dir = temp_dir.path().join("vibe-home");
        std::fs::create_dir_all(&home_dir).unwrap();
        std::fs::write(home_dir.join("app-logs"), b"occupied").unwrap();

        let config = Config::from_test_sources(
            "127.0.0.1",
            Some(0),
            None,
            Some(home_dir.as_os_str().to_owned()),
        )
        .unwrap();

        let error = LogRuntime::new(&config).unwrap_err();
        assert!(matches!(error, ServerError::CreateLogsDir(_)));
    }
}
