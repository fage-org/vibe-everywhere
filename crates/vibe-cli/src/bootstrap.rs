use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use clap::{CommandFactory, Parser};
use serde_json::{Value, json};
use tokio::sync::mpsc;
use uuid::Uuid;
use vibe_agent::{
    api::{DecryptedMachine, DecryptedSession},
    encryption::{encode_base64, encrypt_json},
};
use vibe_wire::SessionTurnEndStatus;

use crate::{
    agent::{
        adapters::{normalize_output_line, render_event_for_output},
        core::{ProviderKind, ProviderRunRequest},
        registry::ProviderRegistry,
    },
    api::CliApiClient,
    auth::{
        AuthRequestResponse, PendingTerminalAuth, auth_logout, auth_status, complete_terminal_auth,
        poll_until_authorized, render_auth_qr, request_terminal_auth,
    },
    commands::{
        AuthCommand, Cli, Commands, ConnectCommand, DaemonCommand, ProviderCommand, SandboxCommand,
        SessionCommand,
    },
    config::Config,
    credentials::require_credentials,
    daemon::{
        daemon_install_status, daemon_list_sessions, daemon_notify_session_started, daemon_status,
        daemon_stop_session, install_daemon, run_daemon_service, start_daemon, stop_daemon,
        uninstall_daemon,
    },
    persistence::{
        LocalSessionState, now_ms, read_settings, remove_local_session, resolve_local_session,
        save_local_session, write_settings,
    },
    sandbox::{SandboxManager, current_mode, disable as disable_sandbox, set_mode},
    transport::default_transport::DefaultTransport,
    ui::display::{format_json, print_heading},
    utils::machine_metadata::{build_machine_metadata, detect_resume_support},
};

#[derive(Debug)]
pub struct BootstrapContext {
    pub cli: Cli,
    pub config: Config,
}

impl BootstrapContext {
    pub fn from_args() -> Result<Self> {
        Ok(Self {
            cli: Cli::parse(),
            config: Config::load()?,
        })
    }
}

pub fn command() -> clap::Command {
    Cli::command()
}

pub async fn run_cli() -> Result<u8> {
    dispatch(BootstrapContext::from_args()?).await
}

pub async fn dispatch(context: BootstrapContext) -> Result<u8> {
    match context.cli.command {
        Commands::Auth { command } => run_auth(&context.config, command).await,
        Commands::Acp { command } => {
            run_provider_command(&context.config, ProviderKind::Acp, command).await
        }
        Commands::Connect { command } => run_connect(&context.config, command).await,
        Commands::Sandbox { command } => run_sandbox(&context.config, command),
        Commands::Daemon { command } => run_daemon(&context.config, command).await,
        Commands::Claude { command } => {
            run_provider_command(&context.config, ProviderKind::Claude, command).await
        }
        Commands::Codex { command } => {
            run_provider_command(&context.config, ProviderKind::Codex, command).await
        }
        Commands::Gemini { command } => {
            run_provider_command(&context.config, ProviderKind::Gemini, command).await
        }
        Commands::Openclaw { command } => {
            run_provider_command(&context.config, ProviderKind::Openclaw, command).await
        }
        Commands::Resume {
            session_id,
            prompt,
            json,
        } => run_resume(&context.config, &session_id, &prompt, json).await,
        Commands::Sessions { command } => run_sessions(&context.config, command).await,
        Commands::Machines { json } => run_machines(&context.config, json).await,
        Commands::DaemonServe => {
            run_daemon_service(context.config).await?;
            Ok(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{BootstrapContext, command, dispatch};
    use crate::{
        commands::{Cli, Commands},
        config::Config,
    };

    #[test]
    fn command_surface_matches_cli_parser() {
        let rendered = command().render_help().to_string();
        assert!(rendered.contains("auth"));
        assert!(rendered.contains("sandbox"));
        assert!(rendered.contains("daemon"));
    }

    #[tokio::test]
    async fn dispatch_surfaces_auth_hint_for_unauthenticated_sessions() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config = Config::from_sources(
            Some("http://127.0.0.1:3005".into()),
            Some("https://app.vibe.engineering".into()),
            Some(temp_dir.path().as_os_str().to_owned()),
            None,
        )
        .unwrap();
        let cli = Cli::try_parse_from(["vibe", "sessions", "list"]).unwrap();

        let error = dispatch(BootstrapContext { cli, config })
            .await
            .unwrap_err();
        assert!(error.to_string().contains("vibe auth login"));
    }

    #[test]
    fn bootstrap_context_holds_cli_and_config() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config = Config::from_sources(
            Some("http://127.0.0.1:3005".into()),
            Some("https://app.vibe.engineering".into()),
            Some(temp_dir.path().as_os_str().to_owned()),
            None,
        )
        .unwrap();
        let cli = Cli::try_parse_from(["vibe", "machines", "--json"]).unwrap();
        let context = BootstrapContext { cli, config };
        assert!(matches!(
            context.cli.command,
            Commands::Machines { json: true }
        ));
        assert!(context.config.home_dir.ends_with(temp_dir.path()));
    }
}

async fn run_auth(config: &Config, command: AuthCommand) -> Result<u8> {
    match command {
        AuthCommand::Login { json } => {
            let client = reqwest::Client::new();
            let pending = PendingTerminalAuth::new();
            let initial = request_terminal_auth(&client, &config, &pending).await?;

            let authorized = match initial {
                AuthRequestResponse::Authorized { token, response } => {
                    crate::auth::AuthorizedTerminalAuth { token, response }
                }
                AuthRequestResponse::Requested => {
                    let qr = render_auth_qr(&pending)?;
                    if json {
                        eprintln!(
                            "{}",
                            json!({
                                "status": "pending",
                                "publicKey": pending.public_key_base64(),
                                "url": pending.deep_link_url(),
                                "webUrl": pending.web_auth_url(&config),
                                "qr": qr,
                            })
                        );
                    } else {
                        println!("\n{qr}\n## Authentication");
                        println!("- Action: Scan this QR code with the Vibe app");
                        println!("- Public Key: `{}`", pending.public_key_base64());
                        println!("- URL: `{}`", pending.deep_link_url());
                        println!("- Web URL: `{}`\n", pending.web_auth_url(&config));
                    }
                    poll_until_authorized(&client, &config, &pending).await?
                }
            };

            let credentials = complete_terminal_auth(&config, &pending, authorized)?;
            let public_key = encode_base64(&credentials.public_key());
            if json {
                println!(
                    "{}",
                    format_json(&json!({
                        "status": "authenticated",
                        "publicKey": public_key,
                    }))
                );
            } else {
                println!("{}", print_heading("Authentication"));
                println!("- Status: Authenticated");
                println!("- Public Key: `{}`", public_key);
            }
            Ok(0)
        }
        AuthCommand::Logout { json } => {
            auth_logout(&config)?;
            if json {
                println!(
                    "{}",
                    format_json(&json!({
                        "status": "loggedOut",
                        "credentialsCleared": true,
                    }))
                );
            } else {
                println!("{}", print_heading("Authentication"));
                println!("- Status: Logged out");
            }
            Ok(0)
        }
        AuthCommand::Status { json } => {
            let payload = match auth_status(&config) {
                Some(credentials) => json!({
                    "status": "authenticated",
                    "publicKey": encode_base64(&credentials.public_key()),
                }),
                None => json!({
                    "status": "notAuthenticated",
                    "action": "Run `vibe auth login` to authenticate.",
                }),
            };
            if json {
                println!("{}", format_json(&payload));
            } else if payload["status"] == "authenticated" {
                println!("{}", print_heading("Authentication"));
                println!("- Status: Authenticated");
                println!(
                    "- Public Key: `{}`",
                    payload["publicKey"].as_str().unwrap_or("-")
                );
            } else {
                println!("{}", print_heading("Authentication"));
                println!("- Status: Not authenticated");
                println!("- Action: Run `vibe auth login` to authenticate.");
            }
            Ok(0)
        }
        AuthCommand::Connect { command } => run_connect(config, command).await,
    }
}

async fn run_connect(config: &Config, command: ConnectCommand) -> Result<u8> {
    let credentials = require_credentials(config)?;
    let api = CliApiClient::new(config.clone(), credentials);
    match command {
        ConnectCommand::Register {
            vendor,
            token,
            json,
        } => {
            api.register_vendor_token(&vendor, &token).await?;
            let payload = json!({ "success": true, "vendor": vendor });
            if json {
                println!("{}", format_json(&payload));
            } else {
                println!("{}", print_heading("Connect"));
                println!("- Vendor: {}", payload["vendor"].as_str().unwrap_or("-"));
                println!("- Registered: yes");
            }
        }
        ConnectCommand::Status { json } => {
            let tokens = api.list_vendor_tokens().await?;
            let payload = tokens
                .into_iter()
                .map(|vendor| json!({ "vendor": vendor, "connected": true }))
                .collect::<Vec<_>>();
            if json {
                println!("{}", format_json(&payload));
            } else {
                println!("{}", print_heading("Connect"));
                if payload.is_empty() {
                    println!("- Tokens: none");
                } else {
                    for token in payload {
                        println!("- {}: connected", token["vendor"].as_str().unwrap_or("-"));
                    }
                }
            }
        }
        ConnectCommand::Delete { vendor, json } => {
            api.delete_vendor_token(&vendor).await?;
            let payload = json!({ "success": true, "vendor": vendor });
            if json {
                println!("{}", format_json(&payload));
            } else {
                println!("{}", print_heading("Connect"));
                println!("- Vendor: {}", payload["vendor"].as_str().unwrap_or("-"));
                println!("- Deleted: yes");
            }
        }
    }
    Ok(0)
}

fn run_sandbox(config: &Config, command: SandboxCommand) -> Result<u8> {
    match command {
        SandboxCommand::Status { json } => {
            let mode = current_mode(&config)?;
            let payload = json!({ "mode": format!("{mode:?}").to_lowercase() });
            if json {
                println!("{}", format_json(&payload));
            } else {
                println!("{}", print_heading("Sandbox"));
                println!("- Mode: {}", payload["mode"].as_str().unwrap_or("-"));
            }
            Ok(0)
        }
        SandboxCommand::Configure { mode, json } => {
            set_mode(&config, mode)?;
            let payload = json!({ "configured": true, "mode": format!("{mode:?}").to_lowercase() });
            if json {
                println!("{}", format_json(&payload));
            } else {
                println!("{}", print_heading("Sandbox"));
                println!("- Configured: yes");
                println!("- Mode: {}", payload["mode"].as_str().unwrap_or("-"));
            }
            Ok(0)
        }
        SandboxCommand::Disable { json } => {
            disable_sandbox(&config)?;
            let payload = json!({ "configured": true, "mode": "disabled" });
            if json {
                println!("{}", format_json(&payload));
            } else {
                println!("{}", print_heading("Sandbox"));
                println!("- Configured: yes");
                println!("- Mode: disabled");
            }
            Ok(0)
        }
    }
}

async fn run_daemon(config: &Config, command: DaemonCommand) -> Result<u8> {
    match command {
        DaemonCommand::Install { json } => {
            let install = install_daemon(&config)?;
            let payload = json!({
                "installed": true,
                "launcherPath": install.launcher_path,
                "executablePath": install.executable_path,
            });
            if json {
                println!("{}", format_json(&payload));
            } else {
                println!("{}", print_heading("Daemon"));
                println!("- Installed: yes");
                println!(
                    "- Launcher: {}",
                    payload["launcherPath"].as_str().unwrap_or("-")
                );
            }
            Ok(0)
        }
        DaemonCommand::Uninstall { json } => {
            let removed = uninstall_daemon(&config).await?;
            let payload = json!({ "uninstalled": removed });
            if json {
                println!("{}", format_json(&payload));
            } else {
                println!("{}", print_heading("Daemon"));
                println!("- Uninstalled: {}", if removed { "yes" } else { "no" });
            }
            Ok(0)
        }
        DaemonCommand::Start { json } => {
            let _ = require_credentials(&config)?;
            let state = start_daemon(&config).await?;
            let payload = json!({
                "running": true,
                "pid": state.pid,
                "startedAt": state.started_at,
                "installed": daemon_install_status(&config)?.is_some(),
            });
            if json {
                println!("{}", format_json(&payload));
            } else {
                println!("{}", print_heading("Daemon"));
                println!("- Status: running");
                println!("- PID: {}", state.pid);
            }
            Ok(0)
        }
        DaemonCommand::List { json } => {
            let sessions = daemon_list_sessions(&config).await?;
            if json {
                println!("{}", format_json(&sessions));
            } else {
                println!("{}", print_heading("Daemon Sessions"));
                if sessions.is_empty() {
                    println!("- Sessions: none");
                } else {
                    for session in sessions {
                        println!("- {} ({})", session.session_id, session.provider);
                    }
                }
            }
            Ok(0)
        }
        DaemonCommand::StopSession { session_id, json } => {
            let stopped = daemon_stop_session(&config, &session_id).await?;
            let payload = json!({ "sessionId": session_id, "stopped": stopped });
            if json {
                println!("{}", format_json(&payload));
            } else {
                println!("{}", print_heading("Daemon"));
                println!("- Session Stopped: {}", if stopped { "yes" } else { "no" });
            }
            Ok(0)
        }
        DaemonCommand::Stop { json } => {
            let stopped = stop_daemon(&config).await?;
            let payload = json!({ "stopped": stopped });
            if json {
                println!("{}", format_json(&payload));
            } else {
                println!("{}", print_heading("Daemon"));
                println!("- Stopped: {}", if stopped { "yes" } else { "no" });
            }
            Ok(0)
        }
        DaemonCommand::Status { json } => {
            let state = daemon_status(&config)?;
            let installed = daemon_install_status(&config)?.is_some();
            let payload = match state {
                Some(state) => json!({
                    "running": true,
                    "installed": installed,
                    "pid": state.pid,
                    "startedAt": state.started_at,
                    "heartbeatAt": state.heartbeat_at,
                }),
                None => json!({ "running": false, "installed": installed }),
            };
            if json {
                println!("{}", format_json(&payload));
            } else {
                println!("{}", print_heading("Daemon"));
                println!(
                    "- Status: {}",
                    if payload["running"] == Value::Bool(true) {
                        "running"
                    } else {
                        "stopped"
                    }
                );
                if let Some(pid) = payload["pid"].as_u64() {
                    println!("- PID: {pid}");
                }
            }
            Ok(0)
        }
    }
}

async fn run_provider_command(
    config: &Config,
    kind: ProviderKind,
    command: ProviderCommand,
) -> Result<u8> {
    match command {
        ProviderCommand::Run {
            tag,
            path,
            prompt,
            json,
        } => {
            ensure_sandbox_supported(config, kind)?;
            let credentials = require_credentials(config)?;
            let api = CliApiClient::new(config.clone(), credentials);
            let working_dir = path.unwrap_or(std::env::current_dir()?.display().to_string());
            let machine_id = ensure_machine_id(config)?;
            let machine = api
                .create_or_load_machine(&machine_id, &machine_metadata(config), None)
                .await?;
            ensure_machine_metadata_current(&api, machine, &machine_metadata(config)).await?;
            let session = api
                .create_session(
                    &tag,
                    &json!({
                        "tag": tag,
                        "path": working_dir,
                        "host": local_hostname(),
                        "provider": kind.as_str(),
                        "machineId": machine_id,
                    }),
                )
                .await?;
            let provider_session_id = Uuid::new_v4().to_string();
            let local_state = LocalSessionState {
                id: session.id.clone(),
                provider: kind.as_str().into(),
                provider_session_id: provider_session_id.clone(),
                server_session_id: session.id.clone(),
                encryption_key: Some(encode_base64(&session.encryption.key)),
                encryption_variant: match session.encryption.variant {
                    vibe_agent::encryption::EncryptionVariant::Legacy => "legacy".into(),
                    vibe_agent::encryption::EncryptionVariant::DataKey => "dataKey".into(),
                },
                tag,
                working_dir: working_dir.clone(),
                created_at: now_ms(),
                updated_at: now_ms(),
            };
            save_local_session(&config, &local_state)?;
            let output =
                run_provider_cycle(config, &api, kind, &session, &local_state, &prompt, false)
                    .await?;

            if json {
                println!(
                    "{}",
                    format_json(&json!({
                        "provider": kind.as_str(),
                        "sessionId": session.id,
                        "providerSessionId": provider_session_id,
                        "path": working_dir,
                        "output": output,
                    }))
                );
            } else {
                println!(
                    "{}",
                    print_heading(&format!("{} Run", kind.as_str().to_ascii_uppercase()))
                );
                println!("- Session ID: `{}`", session.id);
                println!("- Provider Session ID: `{}`", provider_session_id);
                println!("- Path: {}", working_dir);
            }
            Ok(0)
        }
    }
}

async fn run_resume(config: &Config, session_id: &str, prompt: &str, json: bool) -> Result<u8> {
    let credentials = require_credentials(config)?;
    let api = CliApiClient::new(config.clone(), credentials);
    let local_state = resolve_local_session(config, session_id)?;
    let session = match resolve_remote_session(&api, &local_state.server_session_id).await {
        Ok(session) => session,
        Err(error) => {
            let _ = remove_local_session(config, &local_state.id);
            return Err(anyhow!(
                "{error}. Removed stale local session state `{}`.",
                local_state.id
            ));
        }
    };
    let kind = provider_kind_from_name(&local_state.provider)?;
    ensure_sandbox_supported(config, kind)?;
    let output =
        run_provider_cycle(config, &api, kind, &session, &local_state, prompt, true).await?;

    if json {
        println!(
            "{}",
            format_json(&json!({
                "provider": local_state.provider,
                "sessionId": session.id,
                "providerSessionId": local_state.provider_session_id,
                "resumed": true,
                "output": output,
            }))
        );
    } else {
        println!("{}", print_heading("Session Resumed"));
        println!("- Session ID: `{}`", session.id);
        println!(
            "- Provider Session ID: `{}`",
            local_state.provider_session_id
        );
    }
    Ok(0)
}

async fn run_sessions(config: &Config, command: SessionCommand) -> Result<u8> {
    let credentials = require_credentials(config)?;
    let api = CliApiClient::new(config.clone(), credentials.clone());
    match command {
        SessionCommand::List { active, json } => {
            let sessions = if active {
                api.list_active_sessions().await?
            } else {
                api.list_sessions().await?
            };
            if json {
                println!("{}", format_json(&sessions));
            } else {
                println!("{}", print_heading("Sessions"));
                for session in sessions {
                    println!(
                        "- {} ({})",
                        session.id,
                        if session.active { "active" } else { "inactive" }
                    );
                }
            }
        }
        SessionCommand::Status { session_id, json } => {
            let session = resolve_remote_session(&api, &session_id).await?;
            if json {
                println!("{}", format_json(&session));
            } else {
                println!("{}", print_heading("Session"));
                println!("- ID: `{}`", session.id);
                println!("- Active: {}", if session.active { "yes" } else { "no" });
            }
        }
        SessionCommand::History { session_id, json } => {
            let session = resolve_remote_session(&api, &session_id).await?;
            let messages = api.get_session_messages(&session).await?;
            if json {
                println!("{}", format_json(&messages));
            } else {
                println!("{}", print_heading("History"));
                for message in messages {
                    println!("- {} {}", message.id, message.content);
                }
            }
        }
        SessionCommand::Stop { session_id, json } => {
            let session = resolve_remote_session(&api, &session_id).await?;
            let client = api.create_session_client(&session).await?;
            client
                .wait_for_connect(std::time::Duration::from_secs(10))
                .await?;
            client.send_stop().await?;
            client.close().await;
            if json {
                println!(
                    "{}",
                    format_json(&json!({"sessionId": session.id, "stopped": true}))
                );
            } else {
                println!("{}", print_heading("Session"));
                println!("- Stopped: `{}`", session.id);
            }
        }
        SessionCommand::Wait {
            session_id,
            timeout,
            json,
        } => {
            let session = resolve_remote_session(&api, &session_id).await?;
            let client = api.create_session_client(&session).await?;
            client
                .wait_for_connect(std::time::Duration::from_secs(10))
                .await?;
            client
                .wait_for_idle(std::time::Duration::from_secs(timeout))
                .await?;
            client.close().await;
            if json {
                println!(
                    "{}",
                    format_json(
                        &json!({"sessionId": session.id, "idle": true, "timeoutSeconds": timeout}),
                    )
                );
            } else {
                println!("{}", print_heading("Session"));
                println!("- Idle: `{}`", session.id);
            }
        }
    }
    Ok(0)
}

async fn run_machines(config: &Config, json_output: bool) -> Result<u8> {
    let credentials = require_credentials(config)?;
    let api = CliApiClient::new(config.clone(), credentials);
    let machines = api.list_machines().await?;
    if json_output {
        println!("{}", format_json(&machines));
    } else {
        println!("{}", print_heading("Machines"));
        for machine in machines {
            println!(
                "- {} ({})",
                machine.id,
                if machine.active { "active" } else { "inactive" }
            );
        }
    }
    Ok(0)
}

async fn resolve_remote_session(api: &CliApiClient, value: &str) -> Result<DecryptedSession> {
    let mut matches = api
        .list_sessions()
        .await?
        .into_iter()
        .filter(|session| session.id.starts_with(value))
        .collect::<Vec<_>>();
    match matches.len() {
        0 => bail!("No session found matching \"{}\"", value),
        1 => Ok(matches.pop().expect("one match exists")),
        count => bail!(
            "Ambiguous session \"{}\" matches {} records. Be more specific.",
            value,
            count
        ),
    }
}

fn local_hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "unknown".into())
}

async fn run_provider_cycle(
    config: &Config,
    api: &CliApiClient,
    kind: ProviderKind,
    session: &DecryptedSession,
    local_state: &LocalSessionState,
    prompt: &str,
    resume: bool,
) -> Result<String> {
    let transport = DefaultTransport::new(api, session);
    transport.send_user_prompt(prompt).await?;
    let turn_id = transport.start_agent_turn().await?;

    let busy_state = encode_base64(&encrypt_json(
        &session.encryption.key,
        session.encryption.variant,
        &json!({
            "controlledByUser": false,
            "requests": {
                "runtime": {
                    "tool": "provider",
                    "arguments": {
                        "provider": local_state.provider,
                        "resume": resume,
                    },
                    "createdAt": now_ms(),
                }
            }
        }),
    )?);
    let mut version = api
        .update_agent_state(session, session.agent_state_version, Some(Some(busy_state)))
        .await
        .unwrap_or(session.agent_state_version);

    let sandbox_mode = current_mode(api.config())?;
    let (output_tx, mut output_rx) = mpsc::channel(64);
    let (started_tx, mut started_rx) = mpsc::channel(1);
    let provider_registry = ProviderRegistry::new(config);
    let provider_future = provider_registry.run(
        kind,
        ProviderRunRequest {
            provider_session_id: local_state.provider_session_id.clone(),
            prompt: prompt.to_string(),
            working_dir: local_state.working_dir.clone(),
            resume,
            sandbox_mode,
            output_sender: Some(output_tx),
            started_sender: Some(started_tx),
        },
    );
    tokio::pin!(provider_future);

    let daemon_running = daemon_status(api.config())?.is_some();
    let mut daemon_reported = false;
    let mut provider_pid = None;
    let mut streamed_chunks = Vec::new();
    let mut provider_result = None;
    let mut output_closed = false;
    let mut started_closed = false;

    loop {
        tokio::select! {
            started = started_rx.recv(), if provider_pid.is_none() && !started_closed => {
                if let Some(info) = started {
                    provider_pid = Some(info.provider_pid);
                    if daemon_running && !daemon_reported {
                        let _ = daemon_notify_session_started(
                            api.config(),
                            local_state,
                            Some(std::process::id()),
                            provider_pid,
                        ).await;
                        daemon_reported = true;
                    }
                } else {
                    started_closed = true;
                }
            }
            chunk = output_rx.recv(), if !output_closed => {
                match chunk {
                    Some(chunk) => {
                        let normalized = normalize_output_line(&chunk);
                        if let Some(rendered) = render_event_for_output(&normalized) {
                            streamed_chunks.push(rendered);
                        }
                        transport.send_agent_event(&turn_id, &normalized).await?;
                    }
                    None => {
                        output_closed = true;
                        if provider_result.is_some() {
                            break;
                        }
                    }
                }
            }
            result = &mut provider_future, if provider_result.is_none() => {
                provider_result = Some(result);
                if daemon_running && !daemon_reported {
                    let _ = daemon_notify_session_started(
                        api.config(),
                        local_state,
                        Some(std::process::id()),
                        provider_pid,
                    ).await;
                    daemon_reported = true;
                }
                if output_closed {
                    break;
                }
            }
            else => {
                break;
            }
        }

        if provider_result.is_some() && output_closed {
            break;
        }
    }

    let provider_outcome = match provider_result.expect("provider future should resolve") {
        Ok(result) => {
            let output = if result.output.trim().is_empty() && !streamed_chunks.is_empty() {
                streamed_chunks.join("\n")
            } else {
                result.output
            };
            transport
                .finish_agent_turn(&turn_id, &output, SessionTurnEndStatus::Completed)
                .await?;
            Ok(output)
        }
        Err(error) => {
            let error_text = error.to_string();
            let _ = transport.send_agent_text(&turn_id, &error_text).await;
            let _ = transport
                .finish_agent_turn(&turn_id, &error_text, SessionTurnEndStatus::Failed)
                .await;
            Err(error)
        }
    };

    let idle_state = encode_base64(&encrypt_json(
        &session.encryption.key,
        session.encryption.variant,
        &json!({
            "controlledByUser": false,
            "requests": {}
        }),
    )?);
    let idle_result = api
        .update_agent_state(session, version, Some(Some(idle_state)))
        .await;
    if let Ok(updated_version) = idle_result.as_ref() {
        version = *updated_version;
    }
    let _ = version;

    let transport_result = transport.close().await;

    let output = match provider_outcome {
        Ok(output) => output,
        Err(error) => {
            if let Err(idle_error) = idle_result {
                return Err(anyhow!(
                    "{error}; additionally failed to clear agent state: {idle_error}"
                ));
            }
            if let Err(transport_error) = transport_result {
                return Err(anyhow!(
                    "{error}; additionally failed to flush transport: {transport_error}"
                ));
            }
            return Err(error);
        }
    };

    idle_result?;
    transport_result?;

    save_local_session(
        api.config(),
        &LocalSessionState {
            updated_at: now_ms(),
            ..local_state.clone()
        },
    )?;

    Ok(serde_json::from_value::<String>(json!(output.clone())).unwrap_or(output))
}
fn provider_kind_from_name(value: &str) -> Result<ProviderKind> {
    match value {
        "claude" => Ok(ProviderKind::Claude),
        "codex" => Ok(ProviderKind::Codex),
        "gemini" => Ok(ProviderKind::Gemini),
        "openclaw" => Ok(ProviderKind::Openclaw),
        "acp" => Ok(ProviderKind::Acp),
        other => bail!("Unsupported provider: {other}"),
    }
}

fn ensure_machine_id(config: &Config) -> Result<String> {
    let mut settings = read_settings(config)?;
    let machine_id = settings
        .machine_id
        .clone()
        .unwrap_or_else(|| Uuid::now_v7().to_string());
    if settings.machine_id.as_deref() != Some(machine_id.as_str()) {
        settings.machine_id = Some(machine_id.clone());
        write_settings(config, &settings)?;
    }
    Ok(machine_id)
}

async fn ensure_machine_metadata_current(
    api: &CliApiClient,
    machine: DecryptedMachine,
    metadata: &Value,
) -> Result<()> {
    let machine_sync = api.create_machine_sync_client(machine).await?;
    let sync_result = machine_sync
        .sync_machine_metadata_with_retry(metadata, 3, Duration::from_millis(200))
        .await;
    machine_sync.close().await;
    sync_result?;
    Ok(())
}

fn ensure_sandbox_supported(config: &Config, kind: ProviderKind) -> Result<()> {
    let mode = current_mode(config)?;
    SandboxManager::new(mode).ensure_supported(kind)
}

fn machine_metadata(config: &Config) -> Value {
    build_machine_metadata(config, &local_hostname(), detect_resume_support(config))
}
