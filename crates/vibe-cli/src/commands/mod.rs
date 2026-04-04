use clap::{Parser, Subcommand};

use crate::sandbox::SandboxMode;

#[derive(Debug, Parser)]
#[command(name = "vibe", about = "Vibe local runtime CLI", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    Acp {
        #[command(subcommand)]
        command: ProviderCommand,
    },
    #[command(hide = true)]
    Connect {
        #[command(subcommand)]
        command: ConnectCommand,
    },
    Sandbox {
        #[command(subcommand)]
        command: SandboxCommand,
    },
    Daemon {
        #[command(subcommand)]
        command: DaemonCommand,
    },
    Claude {
        #[command(subcommand)]
        command: ProviderCommand,
    },
    Codex {
        #[command(subcommand)]
        command: ProviderCommand,
    },
    Gemini {
        #[command(subcommand)]
        command: ProviderCommand,
    },
    Openclaw {
        #[command(subcommand)]
        command: ProviderCommand,
    },
    Resume {
        session_id: String,
        #[arg(long)]
        prompt: String,
        #[arg(long)]
        json: bool,
    },
    Sessions {
        #[command(subcommand)]
        command: SessionCommand,
    },
    Machines {
        #[arg(long)]
        json: bool,
    },
    #[command(hide = true, name = "__daemon_serve")]
    DaemonServe,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    Login {
        #[arg(long)]
        json: bool,
    },
    Logout {
        #[arg(long)]
        json: bool,
    },
    Status {
        #[arg(long)]
        json: bool,
    },
    Connect {
        #[command(subcommand)]
        command: ConnectCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConnectCommand {
    Register {
        vendor: String,
        #[arg(long)]
        token: String,
        #[arg(long)]
        json: bool,
    },
    Status {
        #[arg(long)]
        json: bool,
    },
    Delete {
        vendor: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum SandboxCommand {
    Status {
        #[arg(long)]
        json: bool,
    },
    Configure {
        #[arg(long, value_enum, default_value_t = SandboxMode::Workspace)]
        mode: SandboxMode,
        #[arg(long)]
        json: bool,
    },
    Disable {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum DaemonCommand {
    Install {
        #[arg(long)]
        json: bool,
    },
    Uninstall {
        #[arg(long)]
        json: bool,
    },
    Start {
        #[arg(long)]
        json: bool,
    },
    List {
        #[arg(long)]
        json: bool,
    },
    StopSession {
        session_id: String,
        #[arg(long)]
        json: bool,
    },
    Stop {
        #[arg(long)]
        json: bool,
    },
    Status {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum ProviderCommand {
    Run {
        #[arg(long)]
        tag: String,
        #[arg(long)]
        path: Option<String>,
        #[arg(long)]
        prompt: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum SessionCommand {
    List {
        #[arg(long)]
        active: bool,
        #[arg(long)]
        json: bool,
    },
    Status {
        session_id: String,
        #[arg(long)]
        json: bool,
    },
    History {
        session_id: String,
        #[arg(long)]
        json: bool,
    },
    Stop {
        session_id: String,
        #[arg(long)]
        json: bool,
    },
    Wait {
        session_id: String,
        #[arg(long, default_value_t = 300)]
        timeout: u64,
        #[arg(long)]
        json: bool,
    },
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, Commands, ConnectCommand, ProviderCommand};

    #[test]
    fn parses_connect_register_command() {
        let cli = Cli::try_parse_from([
            "vibe",
            "auth",
            "connect",
            "register",
            "anthropic",
            "--token",
            "secret",
            "--json",
        ])
        .unwrap();
        match cli.command {
            Commands::Auth {
                command:
                    super::AuthCommand::Connect {
                        command:
                            ConnectCommand::Register {
                                vendor,
                                token,
                                json,
                            },
                    },
            } => {
                assert_eq!(vendor, "anthropic");
                assert_eq!(token, "secret");
                assert!(json);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_codex_run_command() {
        let cli =
            Cli::try_parse_from(["vibe", "codex", "run", "--tag", "demo", "--prompt", "hello"])
                .unwrap();
        match cli.command {
            Commands::Codex {
                command: ProviderCommand::Run { tag, prompt, .. },
            } => {
                assert_eq!(tag, "demo");
                assert_eq!(prompt, "hello");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_acp_run_command() {
        let cli = Cli::try_parse_from(["vibe", "acp", "run", "--tag", "demo", "--prompt", "hello"])
            .unwrap();
        match cli.command {
            Commands::Acp {
                command: ProviderCommand::Run { tag, prompt, .. },
            } => {
                assert_eq!(tag, "demo");
                assert_eq!(prompt, "hello");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_daemon_list_and_stop_session_commands() {
        let list = Cli::try_parse_from(["vibe", "daemon", "list", "--json"]).unwrap();
        match list.command {
            Commands::Daemon {
                command: super::DaemonCommand::List { json },
            } => assert!(json),
            other => panic!("unexpected command: {other:?}"),
        }

        let stop =
            Cli::try_parse_from(["vibe", "daemon", "stop-session", "session-1", "--json"]).unwrap();
        match stop.command {
            Commands::Daemon {
                command: super::DaemonCommand::StopSession { session_id, json },
            } => {
                assert_eq!(session_id, "session-1");
                assert!(json);
            }
            other => panic!("unexpected command: {other:?}"),
        }

        let install = Cli::try_parse_from(["vibe", "daemon", "install", "--json"]).unwrap();
        match install.command {
            Commands::Daemon {
                command: super::DaemonCommand::Install { json },
            } => assert!(json),
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
