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

/// Calculate exponential backoff duration
///
/// Uses exponential growth with random jitter (+/-20%).
pub fn calculate_backoff(
    min: std::time::Duration,
    max: std::time::Duration,
    retry_count: u32,
) -> std::time::Duration {
    let base = min.as_millis() as f64;
    let multiplier = 2_f64.powi(retry_count as i32 - 1);
    let backoff = base * multiplier;
    let backoff = backoff.min(max.as_millis() as f64);

    // Add random jitter (+/-20%)
    let jitter = backoff * 0.2 * (rand::random::<f64>() - 0.5) * 2.0;
    std::time::Duration::from_millis((backoff + jitter) as u64)
}
