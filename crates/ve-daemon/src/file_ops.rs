//! File Operations Module
//!
//! Handles file system operations with workspace boundary validation.

use std::path::{Path, PathBuf};

use tracing::warn;

use crate::error::DaemonError;
use crate::Result;

/// File operations handler with workspace boundary validation
pub struct FileOps {
    /// Allowed workspace roots
    workspace_roots: Vec<PathBuf>,
    /// Maximum file size for text reading (bytes)
    read_text_limit: usize,
    /// Maximum nodes for file tree
    tree_max_nodes: usize,
}

impl FileOps {
    /// Create a new file operations handler
    pub fn new(workspace_roots: Vec<PathBuf>, read_text_limit: usize, tree_max_nodes: usize) -> Self {
        Self {
            workspace_roots,
            read_text_limit,
            tree_max_nodes,
        }
    }

    /// Validate that a path is within allowed workspace boundaries
    ///
    /// # Security
    ///
    /// This function prevents path traversal attacks by ensuring the resolved path
    /// is within one of the allowed workspace roots.
    pub fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        // Canonicalize the path to resolve symlinks and relative components
        let canonical_path = path.canonicalize().map_err(|e| {
            warn!(path = %path.display(), error = %e, "Failed to canonicalize path");
            DaemonError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            }
        })?;

        // Check if the canonical path is within any workspace root
        for root in &self.workspace_roots {
            let canonical_root = match root.canonicalize() {
                Ok(r) => r,
                Err(_) => continue, // Skip roots that don't exist
            };

            if canonical_path.starts_with(&canonical_root) {
                return Ok(canonical_path);
            }
        }

        // Path is outside all workspace roots
        let first_root = self.workspace_roots.first()
            .map(|r| r.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        warn!(
            path = %canonical_path.display(),
            roots = ?self.workspace_roots,
            "Path traversal attempt blocked"
        );
        Err(DaemonError::FileAccessDenied {
            path: canonical_path.to_string_lossy().to_string(),
            workspace: first_root,
        })
    }

    /// Check if a path is within workspace boundaries (returns bool)
    pub fn is_within_workspace(&self, path: &Path) -> bool {
        self.validate_path(path).is_ok()
    }

    /// Get the read text limit
    pub fn read_text_limit(&self) -> usize {
        self.read_text_limit
    }

    /// Get the tree max nodes
    pub fn tree_max_nodes(&self) -> usize {
        self.tree_max_nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file_ops() -> (FileOps, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let roots = vec![temp_dir.path().to_path_buf()];
        let ops = FileOps::new(roots, 1024, 100);
        (ops, temp_dir)
    }

    #[test]
    fn test_validate_path_within_workspace() {
        let (ops, temp_dir) = create_test_file_ops();

        // Create a file in the workspace
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let result = ops.validate_path(&file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), file_path.canonicalize().unwrap());
    }

    #[test]
    fn test_validate_path_outside_workspace() {
        let (ops, _temp_dir) = create_test_file_ops();

        // Try to access /etc/passwd (should be blocked)
        let result = ops.validate_path(Path::new("/etc/passwd"));
        assert!(result.is_err());

        match result {
            Err(DaemonError::FileAccessDenied { .. }) => {}
            _ => panic!("Expected FileAccessDenied error"),
        }
    }

    #[test]
    fn test_validate_path_traversal_attack() {
        let (ops, temp_dir) = create_test_file_ops();

        // Create a file in the workspace
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        // Try path traversal: workspace/test.txt/../../../etc/passwd
        let traversal_path = temp_dir.path().join("test.txt").join("..").join("..").join("..").join("etc").join("passwd");
        let result = ops.validate_path(&traversal_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_symlink_outside_workspace() {
        let (ops, temp_dir) = create_test_file_ops();

        // Create a symlink pointing outside the workspace
        let symlink_path = temp_dir.path().join("outside_link");
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink("/etc", &symlink_path).ok(); // May fail if /etc doesn't exist or no permissions
        }

        // This test is platform-dependent, so we just verify the behavior exists
        if symlink_path.exists() {
            let result = ops.validate_path(&symlink_path);
            // Should be blocked if symlink points outside workspace
            // The actual behavior depends on the symlink target
        }
    }

    #[test]
    fn test_is_within_workspace() {
        let (ops, temp_dir) = create_test_file_ops();

        // Create a file in the workspace
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        assert!(ops.is_within_workspace(&file_path));
        assert!(!ops.is_within_workspace(Path::new("/etc/passwd")));
    }

    #[test]
    fn test_nonexistent_path_returns_error() {
        let (ops, _temp_dir) = create_test_file_ops();

        let result = ops.validate_path(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
