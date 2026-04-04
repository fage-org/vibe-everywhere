#![allow(dead_code)]

use std::{
    fs,
    net::TcpListener as StdTcpListener,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Output},
};

use tempfile::TempDir;
use tokio::{sync::oneshot, task::JoinHandle};
use vibe_cli::{config::Config as CliConfig, credentials::write_credentials};
use vibe_server::{api::build_router, config::Config as ServerConfig, context::AppContext};

pub struct TestServer {
    pub ctx: AppContext,
    pub server_url: String,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<()>>,
}

impl TestServer {
    pub async fn start() -> Self {
        let std_listener = StdTcpListener::bind("127.0.0.1:0").unwrap();
        let addr = std_listener.local_addr().unwrap();
        std_listener.set_nonblocking(true).unwrap();
        let listener = tokio::net::TcpListener::from_std(std_listener).unwrap();

        let config = ServerConfig {
            host: addr.ip(),
            port: addr.port(),
            master_secret: "wave5-secret".into(),
            ios_up_to_date: ">=1.4.1".into(),
            android_up_to_date: ">=1.4.1".into(),
            ios_store_url: "ios-store".into(),
            android_store_url: "android-store".into(),
            webapp_url: "https://app.vibe.engineering".into(),
        };
        let ctx = AppContext::new(config);
        let app = build_router(ctx.clone());
        let server_url = format!("http://{}", listener.local_addr().unwrap());
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let shutdown_ctx = ctx.clone();
        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .unwrap();
            shutdown_ctx.shutdown().await;
        });

        Self {
            ctx,
            server_url,
            shutdown: Some(shutdown_tx),
            task: Some(task),
        }
    }

    pub fn provision_credentials(&self, home: &TempDir) {
        let cli_config = CliConfig::from_sources(
            Some(self.server_url.clone()),
            Some("https://app.vibe.engineering".into()),
            Some(home.path().as_os_str().to_owned()),
            None,
        )
        .unwrap();
        let account = self.ctx.db().upsert_account_by_public_key("wave5-user");
        let token = self.ctx.auth().create_token(&account.id, None);
        write_credentials(&cli_config, token, [7u8; 32]).unwrap();
    }

    pub async fn shutdown(mut self) {
        if let Some(sender) = self.shutdown.take() {
            let _ = sender.send(());
        }
        if let Some(task) = self.task.take() {
            let _ = task.await;
        }
    }
}

pub fn run_cli(home: &TempDir, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vibe"))
        .args(args)
        .env("VIBE_HOME_DIR", home.path())
        .output()
        .unwrap()
}

pub fn run_cli_with_envs(
    home: &TempDir,
    server_url: &str,
    envs: &[(&str, &str)],
    args: &[&str],
) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vibe"));
    command
        .args(args)
        .env("VIBE_HOME_DIR", home.path())
        .env("VIBE_SERVER_URL", server_url);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().unwrap()
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

pub fn write_executable(path: &Path, body: &str) {
    fs::write(path, body).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::write_executable;

    #[test]
    fn write_executable_creates_an_executable_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("demo.sh");
        write_executable(&path, "#!/usr/bin/env bash\nexit 0\n");

        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.is_file());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            assert_ne!(metadata.permissions().mode() & 0o111, 0);
        }
    }
}
