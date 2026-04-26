//! Utility functions for the WebSocket client.

use std::path::Path;

use super::{DaemonError, Result};

/// Ensure a workspace directory exists, creating it if necessary.
/// Only accepts absolute paths.
pub async fn ensure_workspace_directory(workspace_path: &str) -> Result<()> {
    let trimmed_path = workspace_path.trim();
    if trimmed_path.is_empty() {
        return Err(DaemonError::WorkspaceInvalid {
            path: workspace_path.to_string(),
        });
    }

    let path = Path::new(trimmed_path);
    if !path.is_absolute() {
        return Err(DaemonError::WorkspaceInvalid {
            path: trimmed_path.to_string(),
        });
    }

    let exists = tokio::fs::try_exists(path)
        .await
        .map_err(|_| DaemonError::WorkspaceInvalid {
            path: trimmed_path.to_string(),
        })?;
    if exists {
        let is_dir = path.is_dir();
        return if is_dir {
            Ok(())
        } else {
            Err(DaemonError::WorkspaceInvalid {
                path: trimmed_path.to_string(),
            })
        };
    }

    tokio::fs::create_dir_all(path)
        .await
        .map_err(|_| DaemonError::WorkspaceInvalid {
            path: trimmed_path.to_string(),
        })?;

    let is_dir = path.is_dir();
    if is_dir {
        Ok(())
    } else {
        Err(DaemonError::WorkspaceInvalid {
            path: trimmed_path.to_string(),
        })
    }
}

/// Calculate exponential backoff duration with full jitter.
///
/// Uses AWS-style full jitter: `random(0, min(max, base * 2^(retry-1)))`.
/// This distributes retries uniformly across the entire backoff window,
/// preventing thundering herd in large-scale deployments.
pub fn calculate_backoff(
    min: std::time::Duration,
    max: std::time::Duration,
    retry_count: u32,
) -> std::time::Duration {
    let base = min.as_millis() as f64;
    let multiplier = 2_f64.powi(retry_count as i32 - 1);
    let capped = (base * multiplier).min(max.as_millis() as f64);
    let jittered = rand::random::<f64>() * capped;
    std::time::Duration::from_millis(jittered as u64)
}
