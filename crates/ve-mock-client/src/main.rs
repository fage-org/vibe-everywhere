//! Mock Client for Vibe-Remote Integration Testing
//!
//! CLI tool that launches real ve-server + ve-daemon and runs integration test flows.

use clap::Parser;
use std::sync::Arc;
use ve_mock_client::flows::{FlowRegistry, FlowResult};
use ve_mock_client::reporter::{OutputFormat, Reporter};
use ve_mock_client::test_context::TestContext;

#[derive(Parser, Debug)]
#[command(
    name = "ve-mock-client",
    about = "Integration test client for ve-server + ve-daemon"
)]
struct Args {
    /// Comma-separated list of flows to run (default: all)
    #[arg(long)]
    flows: Option<String>,

    /// Remote mode — connect to existing server
    #[arg(long)]
    remote: bool,

    /// Remote server URL (required with --remote)
    #[arg(long, requires = "remote")]
    server_url: Option<String>,

    /// Remote daemon host name (required with --remote)
    #[arg(long, requires = "remote")]
    host_name: Option<String>,

    /// Remote daemon token (required with --remote)
    #[arg(long, requires = "remote")]
    daemon_token: Option<String>,

    /// Remote daemon host ID (optional with --remote; auto-detected if omitted)
    #[arg(long)]
    host_id: Option<String>,

    /// Output format: text or json
    #[arg(long, default_value = "text")]
    output: String,

    /// Skip flows that require Claude Code agent
    #[arg(long)]
    skip_agent: bool,

    /// Number of flows to run concurrently (default: 1 = sequential)
    #[arg(long, default_value = "1")]
    concurrency: usize,
}

struct FlowTaskArgs {
    remote: bool,
    server_url: Option<String>,
    host_name: Option<String>,
    daemon_token: Option<String>,
    host_id: Option<String>,
    skip_agent: bool,
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

    let flow_indices: Vec<usize> = match &selected_flows {
        Some(ids) => all_ids
            .iter()
            .enumerate()
            .filter(|(_, id)| ids.contains(id))
            .map(|(i, _)| i)
            .collect(),
        None => (0..all_ids.len()).collect(),
    };

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
        remote: args.remote,
        server_url: args.server_url,
        host_name: args.host_name,
        daemon_token: args.daemon_token,
        host_id: args.host_id,
        skip_agent: args.skip_agent,
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
            let flow_requires_agent = flow.requires_agent;
            let run_fn = flow.run_fn;
            let sem = Arc::clone(&semaphore);
            let args = FlowTaskArgs {
                remote: task_args.remote,
                server_url: task_args.server_url.clone(),
                host_name: task_args.host_name.clone(),
                daemon_token: task_args.daemon_token.clone(),
                host_id: task_args.host_id.clone(),
                skip_agent: task_args.skip_agent,
            };

            tokio::spawn(async move {
                let _permit = sem.acquire().await.expect("semaphore closed");

                if args.skip_agent && flow_requires_agent {
                    return FlowResult::skipped(&flow_id, "skipped by --skip-agent");
                }

                tracing::info!("=== Running {} ===", flow_id);

                let result = if args.remote {
                    let server_url = args.server_url.clone().expect("server_url required");
                    let host_name = args.host_name.clone().expect("host_name required");
                    let daemon_token = args.daemon_token.clone().expect("daemon_token required");
                    let host_id = args
                        .host_id
                        .as_ref()
                        .and_then(|s| uuid::Uuid::parse_str(s).ok());
                    let ctx = TestContext::new_remote(server_url, host_name, daemon_token, host_id)
                        .expect("failed to create remote context");
                    run_fn(Arc::new(ctx)).await
                } else {
                    match TestContext::new_integration().await {
                        Ok(ctx) => run_fn(Arc::new(ctx)).await,
                        Err(e) => FlowResult::fail(&flow_id, &format!("setup failed: {e}")),
                    }
                };

                result
            })
        })
        .collect();

    futures_util::future::join_all(handles)
        .await
        .into_iter()
        .map(|h| h.expect("flow task panicked"))
        .collect()
}
