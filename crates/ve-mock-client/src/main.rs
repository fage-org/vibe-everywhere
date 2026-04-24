//! Mock Client for Vibe-Remote Integration Testing
//!
//! CLI tool that launches real ve-server + ve-daemon and runs integration test flows.

use clap::{Parser, ValueEnum};
use std::sync::Arc;
use ve_mock_client::flows::{FlowRegistry, FlowResult};
use ve_mock_client::reporter::{OutputFormat, Reporter};
use ve_mock_client::test_context::TestContext;

type FlowFuture = std::pin::Pin<Box<dyn std::future::Future<Output = FlowResult> + Send>>;
type FlowRunner = fn(Arc<TestContext>) -> FlowFuture;
const RELEASE_PROFILE_FLOWS: &[&str] = &[
    "f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8", "f9", "f10", "f11", "f12", "f13",
];

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum FlowProfile {
    Default,
    Release,
}

#[derive(Parser, Debug)]
#[command(
    name = "ve-mock-client",
    about = "Integration test client for ve-server + ve-daemon"
)]
struct Args {
    /// Comma-separated list of flows to run (default: all)
    #[arg(long)]
    flows: Option<String>,

    /// Flow profile: `default` runs the standard suite, `release` adds a real-agent smoke gate.
    #[arg(long, value_enum, default_value_t = FlowProfile::Default)]
    profile: FlowProfile,

    /// Remote mode — connect to existing server
    #[arg(long)]
    remote: bool,

    /// Remote server URL (required with --remote)
    #[arg(long, requires = "remote")]
    server_url: Option<String>,

    /// Remote daemon host name (required with --remote)
    #[arg(long, requires = "remote")]
    host_name: Option<String>,

    /// Remote client token (required with --remote)
    #[arg(long, requires = "remote")]
    client_token: Option<String>,

    /// Remote daemon host ID (optional with --remote; auto-detected if omitted)
    #[arg(long)]
    host_id: Option<String>,

    /// Integration-mode database URL override; also read from VE_MOCK_CLIENT_DATABASE_URL
    #[arg(long)]
    database_url: Option<String>,

    /// Output format: text or json
    #[arg(long, default_value = "text")]
    output: String,

    /// Skip flows that require Claude Code agent
    #[arg(long)]
    skip_agent: bool,

    /// Use real Claude Code agent instead of mock mode (requires `claude` CLI installed)
    #[arg(long)]
    real_agent: bool,

    /// Number of flows to run concurrently (default: 1 = sequential)
    #[arg(long, default_value = "1")]
    concurrency: usize,
}

struct FlowTaskArgs {
    profile: FlowProfile,
    remote: bool,
    server_url: Option<String>,
    host_name: Option<String>,
    client_token: Option<String>,
    host_id: Option<String>,
    database_url: Option<String>,
    skip_agent: bool,
    real_agent: bool,
}

impl Clone for FlowTaskArgs {
    fn clone(&self) -> Self {
        Self {
            profile: self.profile,
            remote: self.remote,
            server_url: self.server_url.clone(),
            host_name: self.host_name.clone(),
            client_token: self.client_token.clone(),
            host_id: self.host_id.clone(),
            database_url: self.database_url.clone(),
            skip_agent: self.skip_agent,
            real_agent: self.real_agent,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new("info,ve_mock_client=debug")
            }),
        )
        .init();

    let args = Args::parse();

    let selected_flows: Option<Vec<String>> = args
        .flows
        .as_ref()
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect());

    let output_format = match args.output.as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Text,
    };
    let reporter = Reporter::new(output_format);

    let registry = Arc::new(FlowRegistry::new());
    let all_ids: Vec<String> = registry.list().iter().map(|f| f.id.clone()).collect();

    let flow_indices = resolve_flow_indices(
        &all_ids,
        selected_flows,
        args.profile,
        args.real_agent,
        args.skip_agent,
    )?;

    let concurrency = args.concurrency.max(1);
    tracing::info!(
        "Running {} flow(s) with concurrency={}",
        flow_indices.len(),
        concurrency
    );

    if concurrency > 1 && !args.remote {
        tracing::warn!(
            "Integration mode with concurrency > 1: each flow spawns its own server + daemon subprocess. \
             This is resource-intensive. Consider using --remote for concurrent testing."
        );
    }

    let task_args = FlowTaskArgs {
        profile: args.profile,
        remote: args.remote,
        server_url: args.server_url,
        host_name: args.host_name,
        client_token: args.client_token,
        host_id: args.host_id,
        database_url: args
            .database_url
            .or_else(|| std::env::var("VE_MOCK_CLIENT_DATABASE_URL").ok()),
        skip_agent: args.skip_agent,
        real_agent: args.real_agent,
    };

    let results = run_flows_concurrent(&registry, &flow_indices, &task_args, concurrency).await;

    reporter.print(&results);

    let failed = results.iter().filter(|r| r.status == "FAIL").count();
    if failed > 0 {
        anyhow::bail!("{failed} flow(s) failed");
    }

    Ok(())
}

async fn run_flows_concurrent(
    registry: &Arc<FlowRegistry>,
    flow_indices: &[usize],
    task_args: &FlowTaskArgs,
    concurrency: usize,
) -> Vec<FlowResult> {
    let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));
    let flows = registry.list();

    let handles: Vec<_> = flow_indices
        .iter()
        .map(|idx| {
            let flow = &flows[*idx];
            let flow_id = flow.id.clone();
            let run_fn = flow.run_fn;
            let flow_requires_agent = flow.requires_agent;
            let flow_requires_real_agent = flow.requires_real_agent;
            let sem = Arc::clone(&semaphore);
            let args = task_args.clone();

            tokio::spawn(async move {
                let _permit = sem.acquire().await.expect("semaphore closed");

                let result = run_flow(
                    flow_id.clone(),
                    flow_requires_agent,
                    flow_requires_real_agent,
                    run_fn,
                    args,
                )
                .await;

                (flow_id, result)
            })
        })
        .collect();

    futures_util::future::join_all(handles)
        .await
        .into_iter()
        .map(|handle| match handle {
            Ok((_, result)) => result,
            Err(err) => FlowResult::fail("unknown", &format!("flow task panicked: {err}")),
        })
        .collect()
}

async fn run_flow(
    flow_id: String,
    flow_requires_agent: bool,
    flow_requires_real_agent: bool,
    run_fn: FlowRunner,
    args: FlowTaskArgs,
) -> FlowResult {
    if args.skip_agent && flow_requires_agent {
        return FlowResult::skipped(&flow_id, "skipped by --skip-agent");
    }

    if !args.real_agent && flow_requires_real_agent {
        return FlowResult::skipped(&flow_id, "requires --real-agent");
    }

    tracing::info!("=== Running {} ===", flow_id);

    if args.remote {
        let server_url = match args.server_url {
            Some(value) => value,
            None => return FlowResult::fail(&flow_id, "setup failed: missing --server-url"),
        };
        let host_name = match args.host_name {
            Some(value) => value,
            None => return FlowResult::fail(&flow_id, "setup failed: missing --host-name"),
        };
        let client_token = match args.client_token {
            Some(value) => value,
            None => return FlowResult::fail(&flow_id, "setup failed: missing --client-token"),
        };
        let host_id = match args.host_id {
            Some(raw) => match uuid::Uuid::parse_str(&raw) {
                Ok(value) => Some(value),
                Err(err) => {
                    return FlowResult::fail(
                        &flow_id,
                        &format!("setup failed: invalid --host-id '{raw}': {err}"),
                    )
                }
            },
            None => None,
        };

        match TestContext::new_remote(server_url, host_name, client_token, host_id).await {
            Ok(ctx) => run_fn(Arc::new(ctx)).await,
            Err(err) => FlowResult::fail(&flow_id, &format!("setup failed: {err:#}")),
        }
    } else {
        let use_real_agent =
            should_use_real_agent(args.profile, args.real_agent, flow_requires_real_agent);
        let mock_mode = !use_real_agent;
        match TestContext::new_integration(mock_mode, args.database_url.clone()).await {
            Ok(ctx) => run_fn(Arc::new(ctx)).await,
            Err(err) => FlowResult::fail(&flow_id, &format!("setup failed: {err:#}")),
        }
    }
}

fn should_use_real_agent(
    profile: FlowProfile,
    real_agent_flag: bool,
    flow_requires_real_agent: bool,
) -> bool {
    match profile {
        FlowProfile::Release => flow_requires_real_agent,
        FlowProfile::Default => real_agent_flag,
    }
}

fn resolve_flow_indices(
    all_ids: &[String],
    selected_flows: Option<Vec<String>>,
    profile: FlowProfile,
    real_agent: bool,
    skip_agent: bool,
) -> anyhow::Result<Vec<usize>> {
    let requested_ids = match selected_flows {
        Some(ids) => ids,
        None => match profile {
            FlowProfile::Default => all_ids.to_vec(),
            FlowProfile::Release => {
                if skip_agent {
                    anyhow::bail!("release profile cannot be used with --skip-agent");
                }
                if !real_agent {
                    anyhow::bail!("release profile requires --real-agent");
                }
                RELEASE_PROFILE_FLOWS
                    .iter()
                    .map(|id| (*id).to_string())
                    .collect()
            }
        },
    };

    let mut indices = Vec::new();
    let mut unknown = Vec::new();
    for id in requested_ids {
        match all_ids.iter().position(|candidate| candidate == &id) {
            Some(index) => indices.push(index),
            None => unknown.push(id),
        }
    }

    if !unknown.is_empty() {
        anyhow::bail!("unknown flow id(s): {}", unknown.join(", "));
    }

    Ok(indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_flow(_ctx: Arc<TestContext>) -> FlowFuture {
        Box::pin(async { FlowResult::pass("dummy", 0.0) })
    }

    #[tokio::test]
    async fn remote_setup_failure_returns_fail_instead_of_panicking() {
        let result = run_flow(
            "f1".to_string(),
            false,
            false,
            dummy_flow,
            FlowTaskArgs {
                profile: FlowProfile::Default,
                remote: true,
                server_url: Some("http://127.0.0.1:1".to_string()),
                host_name: Some("missing-host".to_string()),
                client_token: Some("invalid-token".to_string()),
                host_id: None,
                database_url: None,
                skip_agent: false,
                real_agent: false,
            },
        )
        .await;

        assert_eq!(result.status, "FAIL");
        assert!(result.message.contains("setup failed"));
    }

    #[tokio::test]
    async fn invalid_remote_host_id_returns_fail() {
        let result = run_flow(
            "f1".to_string(),
            false,
            false,
            dummy_flow,
            FlowTaskArgs {
                profile: FlowProfile::Default,
                remote: true,
                server_url: Some("http://127.0.0.1:3000".to_string()),
                host_name: Some("host".to_string()),
                client_token: Some("token".to_string()),
                host_id: Some("not-a-uuid".to_string()),
                database_url: None,
                skip_agent: false,
                real_agent: false,
            },
        )
        .await;

        assert_eq!(result.status, "FAIL");
        assert!(result.message.contains("invalid --host-id"));
    }

    #[test]
    fn release_profile_requires_real_agent() {
        let all_ids = vec!["f1".to_string(), "f13".to_string()];

        let error =
            resolve_flow_indices(&all_ids, None, FlowProfile::Release, false, false).unwrap_err();

        assert!(error.to_string().contains("requires --real-agent"));
    }

    #[test]
    fn release_profile_selects_smoke_gate() {
        let all_ids = RELEASE_PROFILE_FLOWS
            .iter()
            .map(|id| (*id).to_string())
            .collect::<Vec<_>>();

        let indices = resolve_flow_indices(&all_ids, None, FlowProfile::Release, true, false)
            .expect("release profile should resolve");

        assert_eq!(indices.len(), RELEASE_PROFILE_FLOWS.len());
    }

    #[test]
    fn unknown_flow_ids_are_rejected() {
        let error = resolve_flow_indices(
            &["f1".to_string()],
            Some(vec!["missing".to_string()]),
            FlowProfile::Default,
            false,
            false,
        )
        .unwrap_err();

        assert!(error.to_string().contains("unknown flow id"));
    }

    #[test]
    fn release_profile_runs_only_smoke_gate_in_real_mode() {
        assert!(!should_use_real_agent(FlowProfile::Release, true, false));
        assert!(should_use_real_agent(FlowProfile::Release, true, true));
        assert!(!should_use_real_agent(FlowProfile::Default, false, true));
        assert!(should_use_real_agent(FlowProfile::Default, true, false));
    }
}
