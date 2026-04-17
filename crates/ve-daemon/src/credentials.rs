//! Local Credentials Storage
//!
//! Manages persistent storage of daemon credentials for authentication.
//!
//! ## Security
//!
//! The `daemon_token` field is masked in Debug output to prevent accidental
//! exposure in logs. Use `expose_token()` to access the actual value.

use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::DaemonError;

/// Result type for credentials operations
type Result<T> = std::result::Result<T, DaemonError>;

/// Locally persisted credentials
///
/// Note: `daemon_token` is masked in Debug output for security.
/// Use [`Credentials::expose_token()`] to access the actual token value.
#[derive(Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// Host ID (UUID string)
    pub host_id: String,

    /// Daemon authentication token (masked in Debug output)
    pub daemon_token: String,

    /// Server URL these credentials are valid for
    pub server_url: String,

    /// When credentials were created
    pub created_at: DateTime<Utc>,
}

// Custom Debug implementation that masks the daemon_token
impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("host_id", &self.host_id)
            .field("daemon_token", &"***REDACTED***")
            .field("server_url", &self.server_url)
            .field("created_at", &self.created_at)
            .finish()
    }
}

impl Credentials {
    /// Create new credentials
    pub fn new(host_id: String, daemon_token: String, server_url: String) -> Self {
        Self {
            host_id,
            daemon_token,
            server_url,
            created_at: Utc::now(),
        }
    }

    /// Expose the daemon token
    ///
    /// This method provides access to the actual token value.
    /// Use this when the token is needed for authentication.
    pub fn expose_token(&self) -> &str {
        &self.daemon_token
    }

    /// Load credentials from file
    ///
    /// Returns `Ok(None)` if the file doesn't exist.
    /// Returns `Err` if the file exists but cannot be parsed.
    pub fn load(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(path).map_err(DaemonError::ConfigRead)?;

        let creds: Self =
            serde_json::from_str(&content).map_err(|_| DaemonError::TokenParse)?;

        Ok(Some(creds))
    }

    /// Save credentials to file
    ///
    /// Creates parent directories if needed.
    /// Sets file permissions to 0o600 (owner read/write only) on Unix.
    pub fn save(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(DaemonError::ConfigRead)?;
        }

        let content =
            serde_json::to_string_pretty(self).map_err(|e| {
                DaemonError::ConfigInvalid(format!("Failed to serialize credentials: {}", e))
            })?;

        std::fs::write(path, content).map_err(DaemonError::ConfigRead)?;

        // Set file permissions (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
                .map_err(DaemonError::ConfigRead)?;
        }

        Ok(())
    }

    /// Delete credentials file
    ///
    /// Returns `Ok(())` even if the file doesn't exist.
    pub fn delete(path: &Path) -> Result<()> {
        if path.exists() {
            std::fs::remove_file(path).map_err(DaemonError::ConfigRead)?;
        }
        Ok(())
    }

    /// Check if credentials file exists
    pub fn exists(path: &Path) -> bool {
        path.exists()
    }

    /// Validate credentials file permissions
    ///
    /// On Unix, checks that the file has 0o600 permissions.
    /// Returns a warning message if permissions are too permissive.
    #[cfg(unix)]
    pub fn check_permissions(path: &Path) -> Result<Option<String>> {
        if !path.exists() {
            return Ok(None);
        }

        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(path).map_err(DaemonError::ConfigRead)?;
        let mode = metadata.permissions().mode();

        // Check if group or others have any permissions
        if mode & 0o077 != 0 {
            Ok(Some(format!(
                "Credentials file {} has permissive permissions ({:#o}), should be 0o600",
                path.display(),
                mode & 0o777
            )))
        } else {
            Ok(None)
        }
    }

    #[cfg(not(unix))]
    pub fn check_permissions(_path: &Path) -> Result<Option<String>> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_credentials_creation() {
        let creds = Credentials::new(
            "host-123".to_string(),
            "token-abc".to_string(),
            "https://example.com".to_string(),
        );

        assert_eq!(creds.host_id, "host-123");
        assert_eq!(creds.daemon_token, "token-abc");
        assert_eq!(creds.server_url, "https://example.com");
        assert!(creds.created_at <= Utc::now());
    }

    #[test]
    fn test_credentials_save_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("credentials.json");

        let creds = Credentials::new(
            "host-456".to_string(),
            "token-xyz".to_string(),
            "https://server.com".to_string(),
        );

        // Save
        creds.save(&path).unwrap();
        assert!(path.exists());

        // Load
        let loaded = Credentials::load(&path).unwrap().unwrap();
        assert_eq!(loaded.host_id, "host-456");
        assert_eq!(loaded.daemon_token, "token-xyz");
        assert_eq!(loaded.server_url, "https://server.com");
    }

    #[test]
    fn test_credentials_load_missing_file() {
        let path = PathBuf::from("/nonexistent/credentials.json");
        let result = Credentials::load(&path).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_credentials_delete() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("credentials.json");

        let creds = Credentials::new(
            "host-789".to_string(),
            "token-del".to_string(),
            "https://test.com".to_string(),
        );

        creds.save(&path).unwrap();
        assert!(path.exists());

        Credentials::delete(&path).unwrap();
        assert!(!path.exists());

        // Delete non-existent file should succeed
        Credentials::delete(&path).unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn test_credentials_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let path = dir.path().join("credentials.json");

        let creds = Credentials::new(
            "host-perm".to_string(),
            "token-perm".to_string(),
            "https://perm.com".to_string(),
        );

        creds.save(&path).unwrap();

        let metadata = std::fs::metadata(&path).unwrap();
        let mode = metadata.permissions().mode() & 0o777;

        assert_eq!(mode, 0o600, "Credentials file should have 0o600 permissions");

        // Check permission validation
        let warning = Credentials::check_permissions(&path).unwrap();
        assert!(warning.is_none(), "Should have no warning for correct permissions");
    }

    #[test]
    fn test_debug_masks_token() {
        let creds = Credentials::new(
            "host-mask".to_string(),
            "super-secret-token-12345".to_string(),
            "https://mask.com".to_string(),
        );

        let debug_output = format!("{:?}", creds);

        // Debug output should NOT contain the actual token
        assert!(
            !debug_output.contains("super-secret-token-12345"),
            "Debug output should not expose the token"
        );

        // Debug output should contain masked indicator
        assert!(
            debug_output.contains("***") || debug_output.contains("REDACTED"),
            "Debug output should show masked token"
        );
    }

    #[test]
    fn test_expose_token_returns_actual_value() {
        let creds = Credentials::new(
            "host-expose".to_string(),
            "my-secret-token".to_string(),
            "https://expose.com".to_string(),
        );

        // expose_token should return the actual token
        assert_eq!(creds.expose_token(), "my-secret-token");
    }
}
