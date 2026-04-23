//! Test fixtures and data generation

use std::path::Path;

use anyhow::Result;
use uuid::Uuid;

/// Generate a unique workspace path within a temp directory
pub fn unique_workspace(temp_dir: &Path) -> String {
    let id = Uuid::new_v4();
    temp_dir
        .join(format!("workspace-{}", id))
        .to_string_lossy()
        .to_string()
}

/// Generate a unique idempotency key
pub fn unique_idempotency_key() -> String {
    Uuid::new_v4().to_string()
}

/// Generate a unique session title
pub fn unique_session_title() -> String {
    format!("test-session-{}", Uuid::new_v4())
}

/// Generate a unique workspace name
pub fn unique_workspace_name() -> String {
    format!("test-workspace-{}", Uuid::new_v4())
}

/// Generate a fake JWT token for testing error paths (invalid/non-existent device)
pub fn fake_token_for_nonexistent_device() -> String {
    // A syntactically valid-looking JWT with a future expiration (2030-01-01)
    // that won't match any real device record.
    "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJkZXZpY2VfaWQiOiIwMDAwMDAwMC0wMDAwLTAwMDAtMDAwMC0wMDAwMDAwMDAwMDAiLCJ0eXBlIjoiY2xpZW50IiwiZXhwIjoxODkzNDU2MDAwfQ.fake_signature".to_string()
}

/// Create a test directory structure with files
pub fn create_test_workspace(path: &str) -> Result<()> {
    std::fs::create_dir_all(path)?;

    // Create some test files
    let src_dir = format!("{}/src", path);
    std::fs::create_dir_all(&src_dir)?;

    std::fs::write(
        format!("{}/README.md", path),
        "# Test Workspace\n\nThis is a test workspace.\n",
    )?;

    std::fs::write(
        format!("{}/src/main.rs", path),
        "fn main() {\n    println!(\"Hello, world!\");\n}\n",
    )?;

    std::fs::write(
        format!("{}/Cargo.toml", path),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )?;

    Ok(())
}
