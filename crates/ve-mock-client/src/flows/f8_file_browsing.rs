//! F8: File browsing

use std::sync::Arc;

use crate::fixtures;
use crate::flows::FlowResult;
use crate::test_context::TestContext;

pub async fn run(ctx: Arc<TestContext>) -> FlowResult {
    let start = std::time::Instant::now();

    let result = run_impl(&ctx).await;

    match result {
        Ok(()) => FlowResult::pass("f8", start.elapsed().as_secs_f64()),
        Err(e) => FlowResult::fail("f8", &e.to_string()),
    }
}

async fn run_impl(ctx: &TestContext) -> anyhow::Result<()> {
    let client = &ctx.client;

    let host_id = ctx
        .host_id
        .ok_or_else(|| anyhow::anyhow!("F8 requires host_id"))?;

    // Step 1: Create workspace with a real directory structure
    let ws_name = fixtures::unique_workspace_name();
    let ws_path = ctx.workspace_path(&ws_name);

    // Create real test files
    tokio::fs::create_dir_all(&ws_path)
        .await
        .map_err(|e| anyhow::anyhow!("create workspace dir: {e}"))?;

    tokio::fs::create_dir_all(format!("{ws_path}/src"))
        .await
        .map_err(|e| anyhow::anyhow!("create src dir: {e}"))?;

    tokio::fs::write(
        format!("{ws_path}/README.md"),
        "# Test Workspace\n\nThis is a test file for F8.",
    )
    .await
    .map_err(|e| anyhow::anyhow!("write README: {e}"))?;

    tokio::fs::write(
        format!("{ws_path}/src/main.rs"),
        "fn main() { println!(\"Hello\"); }",
    )
    .await
    .map_err(|e| anyhow::anyhow!("write main.rs: {e}"))?;

    tokio::fs::write(
        format!("{ws_path}/src/lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }",
    )
    .await
    .map_err(|e| anyhow::anyhow!("write lib.rs: {e}"))?;

    tracing::info!(path = %ws_path, "Test workspace created");

    // Step 2: Create workspace via API
    let created_ws = client
        .create_workspace(host_id, &ws_path, None)
        .await
        .map_err(|e| anyhow::anyhow!("create workspace: {e}"))?;

    let workspace_id = created_ws.workspace_id;

    tracing::info!(%workspace_id, "Workspace registered");

    // Step 3: Get file tree — root
    let tree = client
        .get_file_tree(host_id, workspace_id, None)
        .await
        .map_err(|e| anyhow::anyhow!("get file tree (root): {e}"))?;

    let root = &tree.data.tree;
    let files = root
        .children
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("file tree root missing children: {tree:?}"))?;

    if files.is_empty() {
        anyhow::bail!("Expected file tree to contain entries, got empty array");
    }

    tracing::info!("Root file tree retrieved: {} entries", files.len());

    // Step 4: Get file tree — subdirectory
    let src_tree = client
        .get_file_tree(host_id, workspace_id, Some("src"))
        .await
        .map_err(|e| anyhow::anyhow!("get file tree (src): {e}"))?;

    let src_root = &src_tree.data.tree;
    let src_files = src_root
        .children
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("src file tree missing children: {src_tree:?}"))?;

    let has_main = src_files.iter().any(|f| f.name == "main.rs");
    let has_lib = src_files.iter().any(|f| f.name == "lib.rs");

    if !has_main || !has_lib {
        anyhow::bail!("src directory should contain main.rs and lib.rs, got: {src_files:?}");
    }

    tracing::info!("Subdirectory file tree verified: main.rs and lib.rs found");

    // Step 5: Get file content — README.md
    let readme = client
        .get_file_content(host_id, workspace_id, "README.md")
        .await
        .map_err(|e| anyhow::anyhow!("get file content (README.md): {e}"))?;

    let content = readme.data.content.as_str();

    if !content.contains("Test Workspace") {
        anyhow::bail!("README.md content mismatch: expected 'Test Workspace', got '{content}'");
    }

    tracing::info!("README.md content verified");

    // Step 6: Get file content — src/main.rs
    let main_rs = client
        .get_file_content(host_id, workspace_id, "src/main.rs")
        .await
        .map_err(|e| anyhow::anyhow!("get file content (src/main.rs): {e}"))?;

    let main_content = main_rs.data.content.as_str();

    if !main_content.contains("println") {
        anyhow::bail!("main.rs content mismatch: expected 'println!', got '{main_content}'");
    }

    tracing::info!("src/main.rs content verified");

    // Step 7: Error path — file tree for non-existent workspace
    let bad_ws = uuid::Uuid::new_v4();
    let result = client.get_file_tree(host_id, bad_ws, None).await;
    if result.is_ok() {
        anyhow::bail!("Expected error for file tree with non-existent workspace, but got OK");
    }

    tracing::info!("Error path verified: non-existent workspace rejected");

    // Step 8: Error path — file content for non-existent file
    let result = client
        .get_file_content(host_id, workspace_id, "nonexistent.txt")
        .await;
    if result.is_ok() {
        anyhow::bail!("Expected error for non-existent file content, but got OK");
    }

    tracing::info!("Error path verified: non-existent file rejected");

    // Step 9: Error path — empty path for file content
    let result = client.get_file_content(host_id, workspace_id, "").await;
    if result.is_ok() {
        anyhow::bail!("Expected error for empty file path, but got OK");
    }

    tracing::info!("Error path verified: empty path rejected");

    tracing::info!("F8 complete: file browsing lifecycle verified");

    Ok(())
}
