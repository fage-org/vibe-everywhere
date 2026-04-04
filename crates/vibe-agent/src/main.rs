use std::{process::ExitCode, time::Duration};

use anyhow::{Result, anyhow, bail};
use clap::{Parser, Subcommand};
use hostname::get;
use reqwest::Client as HttpClient;
use serde_json::Value;
use vibe_agent::{
    api::{ApiClient, DecryptedMachine, DecryptedSession},
    auth::{
        AuthRequestResponse, PendingAccountLink, auth_logout, auth_status, complete_account_link,
        poll_until_authorized, render_auth_qr, request_account_link,
    },
    config::Config,
    credentials::require_credentials,
    machine_rpc::{
        SpawnMachineSessionOptions, SpawnMachineSessionResult, SupportedAgent,
        resume_session_on_machine, spawn_session_on_machine,
    },
    output::{
        format_json, format_machine_table, format_message_history, format_session_status,
        format_session_table,
    },
    session::{SessionClient, SessionClientOptions, SessionEvent},
};

#[derive(Debug, Parser)]
#[command(
    name = "vibe-agent",
    about = "CLI client for controlling Vibe agents remotely",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct PromptOutput {
    stdout: String,
    stderr: String,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Manage authentication
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    /// List all machines
    Machines {
        /// Show only active machines
        #[arg(long)]
        active: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// List all sessions
    List {
        /// Show only active sessions
        #[arg(long)]
        active: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Get live session state
    Status {
        /// Session ID or prefix
        session_id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Spawn a new session on a machine
    Spawn {
        /// Machine ID or prefix
        #[arg(long)]
        machine: String,
        /// Working directory path; defaults to the machine home directory
        #[arg(long)]
        path: Option<String>,
        /// Agent to start
        #[arg(long)]
        agent: Option<SupportedAgent>,
        /// Allow creating the directory if it does not exist
        #[arg(long)]
        create_dir: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Resume a session on its original machine
    Resume {
        /// Session ID or prefix
        session_id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Create a new session
    Create {
        /// Session tag
        #[arg(long)]
        tag: String,
        /// Working directory path
        #[arg(long)]
        path: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Send a message to a session
    Send {
        /// Session ID or prefix
        session_id: String,
        /// Message text
        message: String,
        /// Send with permissionMode=yolo
        #[arg(long)]
        yolo: bool,
        /// Wait for agent turn completion
        #[arg(long)]
        wait: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Read message history
    History {
        /// Session ID or prefix
        session_id: String,
        /// Limit number of messages
        #[arg(long)]
        limit: Option<usize>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Stop a session
    Stop {
        /// Session ID or prefix
        session_id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Wait for agent to become idle
    Wait {
        /// Session ID or prefix
        session_id: String,
        /// Timeout in seconds
        #[arg(long, default_value_t = 300)]
        timeout: u64,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum AuthCommands {
    /// Authenticate via QR code
    Login {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Clear stored credentials
    Logout {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show authentication status
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

async fn run() -> Result<u8> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Auth { command } => run_auth(command).await,
        Commands::Machines { active, json } => {
            let config = Config::load()?;
            let credentials = require_credentials(&config)?;
            let api = ApiClient::new(config, credentials);
            let machines = api.list_machines().await?;
            let machines = if active {
                machines
                    .into_iter()
                    .filter(|machine| machine.active)
                    .collect()
            } else {
                machines
            };

            if json {
                println!("{}", format_json(&machines));
            } else {
                println!("{}", format_machine_table(&machines));
            }
            Ok(0)
        }
        Commands::List { active, json } => {
            let config = Config::load()?;
            let credentials = require_credentials(&config)?;
            let api = ApiClient::new(config, credentials);
            let sessions = if active {
                api.list_active_sessions().await?
            } else {
                api.list_sessions().await?
            };

            if json {
                println!("{}", format_json(&sessions));
            } else {
                println!("{}", format_session_table(&sessions));
            }
            Ok(0)
        }
        Commands::Status { session_id, json } => {
            let config = Config::load()?;
            let credentials = require_credentials(&config)?;
            let api = ApiClient::new(config.clone(), credentials.clone());
            let mut session = resolve_session(&api, &session_id).await?;
            let mut live_data = false;
            if let Ok(client) = create_session_client(&config, &credentials, &session).await {
                let mut receiver = client.subscribe();

                if let Ok(Some((metadata, agent_state))) =
                    tokio::time::timeout(Duration::from_secs(3), async {
                        loop {
                            match receiver.recv().await {
                                Ok(SessionEvent::StateChange {
                                    metadata,
                                    agent_state,
                                }) => {
                                    return Some((metadata, agent_state));
                                }
                                Ok(SessionEvent::Error(_)) | Ok(SessionEvent::Disconnected(_)) => {
                                    return None;
                                }
                                Ok(_) => {}
                                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                    return None;
                                }
                            }
                        }
                    })
                    .await
                {
                    if let Some(metadata) = metadata {
                        session.metadata = metadata;
                    }
                    session.agent_state = agent_state;
                    live_data = true;
                }

                client.close().await;
            }

            if json {
                println!("{}", format_json(&session));
            } else {
                if !live_data {
                    println!("> Note: showing cached data (could not get live status).");
                }
                println!("{}", format_session_status(&session));
            }
            Ok(0)
        }
        Commands::Spawn {
            machine,
            path,
            agent,
            create_dir,
            json,
        } => {
            let config = Config::load()?;
            let credentials = require_credentials(&config)?;
            let api = ApiClient::new(config.clone(), credentials.clone());
            let machine = resolve_machine(&api, &machine).await?;
            let directory = resolve_remote_path(path.as_deref(), &machine)?;
            let result = spawn_session_on_machine(
                &config,
                &machine,
                &credentials.token,
                SpawnMachineSessionOptions {
                    directory: directory.clone(),
                    approved_new_directory_creation: create_dir,
                    agent,
                    provider_token: None,
                },
            )
            .await?;

            if json {
                println!(
                    "{}",
                    format_json(&spawn_result_payload(
                        &machine.id,
                        &directory,
                        agent,
                        &result
                    ))
                );
                return Ok(
                    if matches!(result, SpawnMachineSessionResult::Success { .. }) {
                        0
                    } else {
                        1
                    },
                );
            }

            match result {
                SpawnMachineSessionResult::Success { session_id } => {
                    println!(
                        "## Session Spawned\n\n- Machine ID: `{}`\n- Session ID: `{}`\n- Path: {}\n- Agent: {}",
                        machine.id,
                        session_id,
                        directory,
                        agent
                            .map(|agent| serde_json::to_string(&agent)
                                .unwrap_or_else(|_| "\"default\"".into()))
                            .unwrap_or_else(|| "default".into())
                            .trim_matches('"')
                    );
                    Ok(0)
                }
                SpawnMachineSessionResult::RequestToApproveDirectoryCreation { directory } => {
                    bail!(
                        "The directory '{directory}' does not exist. Re-run with --create-dir to allow creating it."
                    )
                }
                SpawnMachineSessionResult::Error { error_message } => Err(anyhow!(error_message)),
            }
        }
        Commands::Resume { session_id, json } => {
            let config = Config::load()?;
            let credentials = require_credentials(&config)?;
            let api = ApiClient::new(config.clone(), credentials.clone());
            let session = resolve_session(&api, &session_id).await?;
            let machine_id = resolve_session_machine_id(&session)?;
            let machine = resolve_machine(&api, &machine_id).await?;
            ensure_machine_can_resume(&machine)?;

            let result =
                resume_session_on_machine(&config, &machine, &credentials.token, &session.id)
                    .await?;
            if json {
                println!(
                    "{}",
                    format_json(&resume_result_payload(&session.id, &machine.id, &result))
                );
                return Ok(
                    if matches!(result, SpawnMachineSessionResult::Success { .. }) {
                        0
                    } else {
                        1
                    },
                );
            }

            match result {
                SpawnMachineSessionResult::Success { session_id } => {
                    println!(
                        "## Session Resumed\n\n- Machine ID: `{}`\n- Source Session ID: `{}`\n- Resumed Session ID: `{}`",
                        machine.id, session.id, session_id
                    );
                    Ok(0)
                }
                SpawnMachineSessionResult::RequestToApproveDirectoryCreation { directory } => {
                    bail!(
                        "Resume unexpectedly requested directory creation for '{directory}'. Resume should reuse the saved path."
                    )
                }
                SpawnMachineSessionResult::Error { error_message } => Err(anyhow!(error_message)),
            }
        }
        Commands::Create { tag, path, json } => {
            let config = Config::load()?;
            let credentials = require_credentials(&config)?;
            let api = ApiClient::new(config, credentials);
            let session = api
                .create_session(
                    &tag,
                    &serde_json::json!({
                        "tag": tag,
                        "path": resolve_local_path(path)?,
                        "host": local_hostname(),
                    }),
                )
                .await?;

            if json {
                println!("{}", format_json(&session));
            } else {
                println!("## Session Created\n\n- Session ID: `{}`", session.id);
            }
            Ok(0)
        }
        Commands::Send {
            session_id,
            message,
            yolo,
            wait,
            json,
        } => {
            let config = Config::load()?;
            let credentials = require_credentials(&config)?;
            let api = ApiClient::new(config.clone(), credentials.clone());
            let session = resolve_session(&api, &session_id).await?;
            let client = create_session_client(&config, &credentials, &session).await?;
            let permission_mode = yolo.then_some("yolo");
            let meta = permission_mode.map(|mode| {
                let mut meta = serde_json::Map::new();
                meta.insert("permissionMode".into(), Value::String(mode.into()));
                meta
            });
            let outcome: Result<(), vibe_agent::session::SessionError> = async {
                client.wait_for_connect(Duration::from_secs(10)).await?;
                let mut receiver = client.subscribe();
                client.send_message(&message, meta).await?;
                if wait {
                    client
                        .wait_for_turn_completion_on(&mut receiver, Duration::from_secs(300))
                        .await?;
                } else {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Ok(())
            }
            .await;
            client.close().await;
            outcome?;

            if json {
                println!(
                    "{}",
                    format_json(&send_result_payload(&session.id, &message, permission_mode))
                );
            } else {
                println!(
                    "## Message Sent\n\n- Session ID: `{}`\n- Permission Mode: {}\n- Waited For Idle: {}",
                    session.id,
                    permission_mode.unwrap_or("default"),
                    if wait { "yes" } else { "no" }
                );
            }
            Ok(0)
        }
        Commands::History {
            session_id,
            limit,
            json,
        } => {
            let config = Config::load()?;
            let credentials = require_credentials(&config)?;
            let api = ApiClient::new(config, credentials);
            let session = resolve_session(&api, &session_id).await?;
            let mut messages = api
                .get_session_messages(&session.id, &session.encryption)
                .await?;
            messages.sort_by_key(|message| message.created_at);
            if let Some(limit) = limit {
                if limit == 0 {
                    bail!("--limit must be a positive integer");
                }
                if messages.len() > limit {
                    messages = messages.split_off(messages.len() - limit);
                }
            }

            if json {
                println!("{}", format_json(&messages));
            } else {
                println!("{}", format_message_history(&messages));
            }
            Ok(0)
        }
        Commands::Stop { session_id, json } => {
            let config = Config::load()?;
            let credentials = require_credentials(&config)?;
            let api = ApiClient::new(config.clone(), credentials.clone());
            let session = resolve_session(&api, &session_id).await?;
            let client = create_session_client(&config, &credentials, &session).await?;
            let outcome: Result<(), vibe_agent::session::SessionError> = async {
                client.wait_for_connect(Duration::from_secs(10)).await?;
                client.send_stop().await?;
                tokio::time::sleep(Duration::from_millis(500)).await;
                Ok(())
            }
            .await;
            client.close().await;
            outcome?;

            if json {
                println!("{}", format_json(&stop_result_payload(&session.id)));
            } else {
                println!("## Session Stopped\n\n- Session ID: `{}`", session.id);
            }
            Ok(0)
        }
        Commands::Wait {
            session_id,
            timeout,
            json,
        } => {
            if timeout == 0 {
                bail!("--timeout must be a positive integer");
            }
            let config = Config::load()?;
            let credentials = require_credentials(&config)?;
            let api = ApiClient::new(config.clone(), credentials.clone());
            let session = resolve_session(&api, &session_id).await?;
            let client = create_session_client(&config, &credentials, &session).await?;
            let outcome: Result<(), vibe_agent::session::SessionError> = async {
                client.wait_for_connect(Duration::from_secs(10)).await?;
                client.wait_for_idle(Duration::from_secs(timeout)).await?;
                Ok(())
            }
            .await;
            client.close().await;
            outcome?;

            if json {
                println!(
                    "{}",
                    format_json(&wait_result_payload(&session.id, timeout))
                );
            } else {
                println!("## Session Idle\n\n- Session ID: `{}`", session.id);
            }
            Ok(0)
        }
    }
}

async fn run_auth(command: AuthCommands) -> Result<u8> {
    let config = Config::load()?;
    match command {
        AuthCommands::Login { json } => {
            let client = HttpClient::new();
            let pending = PendingAccountLink::new();
            let initial = request_account_link(&client, &config, &pending).await?;

            let authorized = match initial {
                AuthRequestResponse::Authorized { token, response } => {
                    vibe_agent::auth::AuthorizedAccountLink { token, response }
                }
                AuthRequestResponse::Requested => {
                    print_auth_login_prompt(json, &pending)?;
                    poll_until_authorized(&client, &config, &pending).await?
                }
            };

            let credentials = complete_account_link(&config, &pending, authorized)?;
            if json {
                println!(
                    "{}",
                    format_json(&auth_login_payload(&credentials_public_key(&credentials)))
                );
            } else {
                println!("## Authentication");
                println!("- Status: Authenticated");
            }
            Ok(0)
        }
        AuthCommands::Logout { json } => {
            auth_logout(&config)?;
            if json {
                println!("{}", format_json(&auth_logout_payload()));
            } else {
                println!("## Authentication");
                println!("- Status: Logged out");
                println!("- Credentials: Cleared");
            }
            Ok(0)
        }
        AuthCommands::Status { json } => {
            if let Some(credentials) = auth_status(&config) {
                let public_key = credentials_public_key(&credentials);
                if json {
                    println!("{}", format_json(&auth_status_payload(Some(&public_key))));
                } else {
                    println!("## Authentication");
                    println!("- Status: Authenticated");
                    println!("- Public Key: `{}`", public_key);
                }
            } else {
                if json {
                    println!("{}", format_json(&auth_status_payload(None)));
                } else {
                    println!("## Authentication");
                    println!("- Status: Not authenticated");
                    println!("- Action: Run `vibe-agent auth login` to authenticate.");
                }
            }
            Ok(0)
        }
    }
}

fn print_auth_login_prompt(json: bool, pending: &PendingAccountLink) -> Result<()> {
    let output = auth_login_prompt_output(json, pending)?;
    if !output.stdout.is_empty() {
        print!("{}", output.stdout);
    }
    if !output.stderr.is_empty() {
        eprint!("{}", output.stderr);
    }
    Ok(())
}

fn auth_login_prompt_output(json: bool, pending: &PendingAccountLink) -> Result<PromptOutput> {
    let prompt = auth_login_prompt_text(pending)?;
    Ok(if json {
        PromptOutput {
            stdout: String::new(),
            stderr: prompt,
        }
    } else {
        PromptOutput {
            stdout: prompt,
            stderr: String::new(),
        }
    })
}

fn auth_login_prompt_text(pending: &PendingAccountLink) -> Result<String> {
    let qr = render_auth_qr(pending)?;
    Ok(format!(
        "\n{qr}\n## Authentication\n- Action: Scan this QR code with the Vibe app\n- Path: Settings -> Account -> Link New Device\n- Public Key: `{}`\n- URL: `{}`\n\n",
        pending.public_key_base64(),
        pending.qr_url()
    ))
}

async fn resolve_session(api: &ApiClient, value: &str) -> Result<DecryptedSession> {
    let sessions = api.list_sessions().await?;
    resolve_by_prefix(sessions, value, "Session ID", |session| &session.id)
}

async fn resolve_machine(api: &ApiClient, value: &str) -> Result<DecryptedMachine> {
    let machines = api.list_machines().await?;
    resolve_by_prefix(machines, value, "Machine ID", |machine| &machine.id)
}

fn resolve_by_prefix<T, F>(items: Vec<T>, value: &str, label: &str, id_fn: F) -> Result<T>
where
    F: Fn(&T) -> &str,
{
    if value.trim().is_empty() {
        bail!("{label} is required");
    }

    let mut matches = items
        .into_iter()
        .filter(|item| id_fn(item).starts_with(value))
        .collect::<Vec<_>>();
    match matches.len() {
        0 => bail!("No {} found matching \"{}\"", label.to_lowercase(), value),
        1 => Ok(matches.pop().expect("one match exists")),
        count => bail!(
            "Ambiguous {} \"{}\" matches {} records. Be more specific.",
            label.to_lowercase(),
            value,
            count
        ),
    }
}

async fn create_session_client(
    config: &Config,
    credentials: &vibe_agent::credentials::Credentials,
    session: &DecryptedSession,
) -> Result<SessionClient> {
    Ok(SessionClient::connect(SessionClientOptions {
        session_id: session.id.clone(),
        encryption_key: session.encryption.key,
        encryption_variant: session.encryption.variant,
        token: credentials.token.clone(),
        server_url: config.socket_url(),
        initial_metadata: Some(session.metadata.clone()),
        initial_metadata_version: session.metadata_version,
        initial_agent_state: session.agent_state.clone(),
        initial_agent_state_version: session.agent_state_version,
    })
    .await?)
}

fn resolve_remote_path(raw_path: Option<&str>, machine: &DecryptedMachine) -> Result<String> {
    let home_dir = machine
        .metadata
        .get("homeDir")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty());
    let path = raw_path.or(home_dir);

    let Some(path) = path else {
        bail!("Machine metadata does not include a home directory. Pass --path explicitly.");
    };

    if path == "~" {
        return home_dir
            .map(ToOwned::to_owned)
            .ok_or_else(|| anyhow!("Machine metadata does not include a home directory, so `~` cannot be resolved. Pass an absolute --path."));
    }

    if let Some(suffix) = path.strip_prefix("~/") {
        let Some(home_dir) = home_dir else {
            bail!(
                "Machine metadata does not include a home directory, so `~/...` cannot be resolved. Pass an absolute --path."
            );
        };
        let normalized_home = home_dir.trim_end_matches(['/', '\\']);
        let separator = if normalized_home.contains('\\') && !normalized_home.contains('/') {
            '\\'
        } else {
            '/'
        };
        let suffix = suffix.replace('/', &separator.to_string());
        return Ok(format!("{normalized_home}{separator}{suffix}"));
    }

    Ok(path.to_string())
}

fn resolve_session_machine_id(session: &DecryptedSession) -> Result<String> {
    session
        .metadata
        .get("machineId")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            anyhow!(
                "Session {} is missing machine metadata and cannot be resumed.",
                session.id
            )
        })
}

fn ensure_machine_can_resume(machine: &DecryptedMachine) -> Result<()> {
    let resume_support = machine
        .metadata
        .get("resumeSupport")
        .and_then(Value::as_object);
    if resume_support
        .and_then(|resume_support| resume_support.get("rpcAvailable"))
        .and_then(Value::as_bool)
        == Some(true)
    {
        return Ok(());
    }
    if resume_support
        .and_then(|resume_support| resume_support.get("happyAgentAuthenticated"))
        .and_then(Value::as_bool)
        == Some(false)
    {
        bail!(
            "Resume is unavailable on this machine. Run `vibe-agent auth login` in that machine environment first."
        );
    }
    bail!("Resume RPC is unavailable on this machine right now.")
}

fn resolve_local_path(path: Option<String>) -> Result<String> {
    match path {
        Some(path) => Ok(path),
        None => Ok(std::env::current_dir()?.display().to_string()),
    }
}

fn local_hostname() -> String {
    get()
        .ok()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "unknown".into())
}

fn credentials_public_key(credentials: &vibe_agent::credentials::Credentials) -> String {
    vibe_agent::encryption::encode_base64(&credentials.content_key_pair.public_key)
}

fn auth_login_payload(public_key: &str) -> Value {
    serde_json::json!({
        "status": "authenticated",
        "publicKey": public_key,
    })
}

fn auth_logout_payload() -> Value {
    serde_json::json!({
        "status": "loggedOut",
        "credentialsCleared": true,
    })
}

fn auth_status_payload(public_key: Option<&str>) -> Value {
    match public_key {
        Some(public_key) => serde_json::json!({
            "status": "authenticated",
            "publicKey": public_key,
        }),
        None => serde_json::json!({
            "status": "notAuthenticated",
            "action": "Run `vibe-agent auth login` to authenticate.",
        }),
    }
}

fn send_result_payload(session_id: &str, message: &str, permission_mode: Option<&str>) -> Value {
    serde_json::json!({
        "sessionId": session_id,
        "message": message,
        "sent": true,
        "permissionMode": permission_mode,
    })
}

fn stop_result_payload(session_id: &str) -> Value {
    serde_json::json!({
        "sessionId": session_id,
        "stopped": true,
    })
}

fn wait_result_payload(session_id: &str, timeout: u64) -> Value {
    serde_json::json!({
        "sessionId": session_id,
        "idle": true,
        "timeoutSeconds": timeout,
    })
}

fn spawn_result_payload(
    machine_id: &str,
    directory: &str,
    agent: Option<SupportedAgent>,
    result: &SpawnMachineSessionResult,
) -> Value {
    let mut payload = serde_json::Map::new();
    payload.insert("machineId".into(), Value::String(machine_id.into()));
    payload.insert("directory".into(), Value::String(directory.into()));
    payload.insert(
        "agent".into(),
        agent
            .map(|agent| serde_json::to_value(agent).unwrap_or(Value::Null))
            .unwrap_or(Value::Null),
    );
    flatten_spawn_result(&mut payload, result);
    Value::Object(payload)
}

fn resume_result_payload(
    source_session_id: &str,
    machine_id: &str,
    result: &SpawnMachineSessionResult,
) -> Value {
    let mut payload = serde_json::Map::new();
    payload.insert(
        "sourceSessionId".into(),
        Value::String(source_session_id.into()),
    );
    payload.insert("machineId".into(), Value::String(machine_id.into()));
    flatten_spawn_result(&mut payload, result);
    Value::Object(payload)
}

fn flatten_spawn_result(
    payload: &mut serde_json::Map<String, Value>,
    result: &SpawnMachineSessionResult,
) {
    match result {
        SpawnMachineSessionResult::Success { session_id } => {
            payload.insert("type".into(), Value::String("success".into()));
            payload.insert("sessionId".into(), Value::String(session_id.clone()));
        }
        SpawnMachineSessionResult::RequestToApproveDirectoryCreation { directory } => {
            payload.insert(
                "type".into(),
                Value::String("requestToApproveDirectoryCreation".into()),
            );
            payload.insert("directory".into(), Value::String(directory.clone()));
        }
        SpawnMachineSessionResult::Error { error_message } => {
            payload.insert("type".into(), Value::String("error".into()));
            payload.insert("errorMessage".into(), Value::String(error_message.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};
    use serde_json::json;

    use super::{
        AuthCommands, Cli, Commands, SupportedAgent, auth_login_payload, auth_login_prompt_output,
        auth_logout_payload, auth_status_payload, credentials_public_key, resume_result_payload,
        send_result_payload, spawn_result_payload, stop_result_payload, wait_result_payload,
    };
    use vibe_agent::auth::PendingAccountLink;
    use vibe_agent::credentials::Credentials;
    use vibe_agent::encryption::derive_content_key_pair;
    use vibe_agent::machine_rpc::SpawnMachineSessionResult;

    #[test]
    fn parses_auth_login_command() {
        let cli = Cli::try_parse_from(["vibe-agent", "auth", "login", "--json"]).unwrap();

        match cli.command {
            Commands::Auth {
                command: AuthCommands::Login { json },
            } => assert!(json),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_spawn_command_with_agent_and_json() {
        let cli = Cli::try_parse_from([
            "vibe-agent",
            "spawn",
            "--machine",
            "machine-1",
            "--path",
            "/tmp/project",
            "--agent",
            "codex",
            "--create-dir",
            "--json",
        ])
        .unwrap();

        match cli.command {
            Commands::Spawn {
                machine,
                path,
                agent,
                create_dir,
                json,
            } => {
                assert_eq!(machine, "machine-1");
                assert_eq!(path.as_deref(), Some("/tmp/project"));
                assert_eq!(agent, Some(SupportedAgent::Codex));
                assert!(create_dir);
                assert!(json);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_send_command_with_wait_and_yolo() {
        let cli = Cli::try_parse_from([
            "vibe-agent",
            "send",
            "session-1",
            "hello",
            "--wait",
            "--yolo",
        ])
        .unwrap();

        match cli.command {
            Commands::Send {
                session_id,
                message,
                wait,
                yolo,
                json,
            } => {
                assert_eq!(session_id, "session-1");
                assert_eq!(message, "hello");
                assert!(wait);
                assert!(yolo);
                assert!(!json);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn wait_command_uses_default_timeout() {
        let cli = Cli::try_parse_from(["vibe-agent", "wait", "session-1", "--json"]).unwrap();

        match cli.command {
            Commands::Wait {
                session_id,
                timeout,
                json,
            } => {
                assert_eq!(session_id, "session-1");
                assert_eq!(timeout, 300);
                assert!(json);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_stop_command_with_json() {
        let cli = Cli::try_parse_from(["vibe-agent", "stop", "session-1", "--json"]).unwrap();

        match cli.command {
            Commands::Stop { session_id, json } => {
                assert_eq!(session_id, "session-1");
                assert!(json);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn auth_login_prompt_uses_stderr_in_json_mode() {
        let pending = PendingAccountLink {
            public_key: [7u8; 32],
            secret_key: [9u8; 32],
        };

        let output = auth_login_prompt_output(true, &pending).unwrap();
        assert!(output.stdout.is_empty());
        assert!(output.stderr.contains("## Authentication"));
        assert!(output.stderr.contains("vibe:///account?"));
        assert!(output.stderr.contains("Link New Device"));
    }

    #[test]
    fn auth_login_prompt_uses_stdout_in_human_mode() {
        let pending = PendingAccountLink {
            public_key: [3u8; 32],
            secret_key: [5u8; 32],
        };

        let output = auth_login_prompt_output(false, &pending).unwrap();
        assert!(output.stderr.is_empty());
        assert!(output.stdout.contains("## Authentication"));
        assert!(output.stdout.contains("vibe:///account?"));
    }

    #[test]
    fn json_payloads_match_wave3_contract() {
        assert_eq!(
            auth_login_payload("pub-key"),
            json!({
                "status": "authenticated",
                "publicKey": "pub-key",
            })
        );
        assert_eq!(
            auth_logout_payload(),
            json!({
                "status": "loggedOut",
                "credentialsCleared": true,
            })
        );
        assert_eq!(
            auth_status_payload(Some("pub-key")),
            json!({
                "status": "authenticated",
                "publicKey": "pub-key",
            })
        );
        assert_eq!(
            auth_status_payload(None),
            json!({
                "status": "notAuthenticated",
                "action": "Run `vibe-agent auth login` to authenticate.",
            })
        );
        assert_eq!(
            send_result_payload("session-1", "hello", Some("yolo")),
            json!({
                "sessionId": "session-1",
                "message": "hello",
                "sent": true,
                "permissionMode": "yolo",
            })
        );
        assert_eq!(
            stop_result_payload("session-1"),
            json!({
                "sessionId": "session-1",
                "stopped": true,
            })
        );
        assert_eq!(
            wait_result_payload("session-1", 300),
            json!({
                "sessionId": "session-1",
                "idle": true,
                "timeoutSeconds": 300,
            })
        );
        assert_eq!(
            spawn_result_payload(
                "machine-1",
                "/tmp/project",
                Some(SupportedAgent::Codex),
                &SpawnMachineSessionResult::Success {
                    session_id: "session-2".into(),
                }
            ),
            json!({
                "machineId": "machine-1",
                "directory": "/tmp/project",
                "agent": "codex",
                "type": "success",
                "sessionId": "session-2",
            })
        );
        assert_eq!(
            resume_result_payload(
                "session-1",
                "machine-1",
                &SpawnMachineSessionResult::Error {
                    error_message: "daemon failed".into(),
                }
            ),
            json!({
                "sourceSessionId": "session-1",
                "machineId": "machine-1",
                "type": "error",
                "errorMessage": "daemon failed",
            })
        );
    }

    #[test]
    fn cli_root_command_exposes_version_metadata() {
        let command = Cli::command();
        assert_eq!(command.get_version(), Some(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn cli_help_includes_command_descriptions() {
        let mut command = Cli::command();
        let mut help = Vec::new();
        command.write_long_help(&mut help).unwrap();
        let help = String::from_utf8(help).unwrap();

        assert!(help.contains("Manage authentication"));
        assert!(help.contains("List all sessions"));
        assert!(help.contains("Get live session state"));
    }

    #[test]
    fn credentials_public_key_uses_persisted_content_key() {
        let secret = [7u8; 32];
        let content_key_pair = derive_content_key_pair(&secret);
        let credentials = Credentials {
            token: "token-1".into(),
            secret,
            content_key_pair,
        };

        assert_eq!(
            credentials_public_key(&credentials),
            vibe_agent::encryption::encode_base64(&credentials.content_key_pair.public_key)
        );
    }
}
