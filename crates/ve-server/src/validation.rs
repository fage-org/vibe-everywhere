//! Input Validation
//!
//! Provides validation helpers for user input fields.

use thiserror::Error;

/// Maximum lengths for various fields
pub const MAX_DEVICE_NAME_LENGTH: usize = 255;
pub const MAX_HOST_NAME_LENGTH: usize = 255;
pub const PAIR_CODE_LENGTH: usize = 6;
pub const MAX_TITLE_LENGTH: usize = 500;
pub const MAX_CONTENT_LENGTH: usize = 100000; // 100KB
pub const MAX_WORKSPACE_PATH_LENGTH: usize = 1024;
pub const MAX_WORKSPACE_DISPLAY_NAME_LENGTH: usize = 255;
pub const MAX_IDEMPOTENCY_KEY_LENGTH: usize = 128;

/// Maximum number of items in a batch operation
pub const MAX_BATCH_DELETE_SIZE: usize = 100;

/// Validation error types
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("{field} cannot be empty")]
    Empty { field: &'static str },

    #[error("{field} exceeds maximum length of {max} characters")]
    TooLong { field: &'static str, max: usize },

    #[error("{field} contains invalid characters")]
    #[allow(dead_code)]
    InvalidChars { field: &'static str },

    #[error("{field} exceeds maximum count of {max}")]
    TooMany { field: &'static str, max: usize },
}

/// Validate device name
pub fn validate_device_name(name: &str) -> Result<(), ValidationError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::Empty {
            field: "device_name",
        });
    }
    if trimmed.len() > MAX_DEVICE_NAME_LENGTH {
        return Err(ValidationError::TooLong {
            field: "device_name",
            max: MAX_DEVICE_NAME_LENGTH,
        });
    }
    Ok(())
}

/// Validate host name
pub fn validate_host_name(name: &str) -> Result<(), ValidationError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::Empty { field: "host_name" });
    }
    if trimmed.len() > MAX_HOST_NAME_LENGTH {
        return Err(ValidationError::TooLong {
            field: "host_name",
            max: MAX_HOST_NAME_LENGTH,
        });
    }
    Ok(())
}

/// Validate pair code
pub fn validate_pair_code(code: &str) -> Result<(), ValidationError> {
    if code.len() != PAIR_CODE_LENGTH {
        return Err(ValidationError::InvalidChars { field: "pair_code" });
    }

    if !code
        .bytes()
        .all(|byte| matches!(byte, b'A'..=b'H' | b'J'..=b'N' | b'P'..=b'Z' | b'2'..=b'9'))
    {
        return Err(ValidationError::InvalidChars { field: "pair_code" });
    }

    Ok(())
}

/// Validate session title
pub fn validate_title(title: &str) -> Result<(), ValidationError> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::Empty { field: "title" });
    }
    if trimmed.len() > MAX_TITLE_LENGTH {
        return Err(ValidationError::TooLong {
            field: "title",
            max: MAX_TITLE_LENGTH,
        });
    }
    Ok(())
}

/// Validate message content
pub fn validate_content(content: &str) -> Result<(), ValidationError> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::Empty { field: "content" });
    }
    if trimmed.len() > MAX_CONTENT_LENGTH {
        return Err(ValidationError::TooLong {
            field: "content",
            max: MAX_CONTENT_LENGTH,
        });
    }
    Ok(())
}

/// Validate workspace path
pub fn validate_workspace_path(path: &str) -> Result<(), ValidationError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::Empty {
            field: "workspace_path",
        });
    }
    if path.len() > MAX_WORKSPACE_PATH_LENGTH {
        return Err(ValidationError::TooLong {
            field: "workspace_path",
            max: MAX_WORKSPACE_PATH_LENGTH,
        });
    }
    Ok(())
}

/// Validate workspace display name
pub fn validate_workspace_display_name(display_name: &str) -> Result<(), ValidationError> {
    let trimmed = display_name.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::Empty {
            field: "workspace_display_name",
        });
    }
    if display_name.len() > MAX_WORKSPACE_DISPLAY_NAME_LENGTH {
        return Err(ValidationError::TooLong {
            field: "workspace_display_name",
            max: MAX_WORKSPACE_DISPLAY_NAME_LENGTH,
        });
    }
    Ok(())
}

/// Validate idempotency key
pub fn validate_idempotency_key(key: &str) -> Result<(), ValidationError> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::Empty {
            field: "idempotency_key",
        });
    }
    if key.len() > MAX_IDEMPOTENCY_KEY_LENGTH {
        return Err(ValidationError::TooLong {
            field: "idempotency_key",
            max: MAX_IDEMPOTENCY_KEY_LENGTH,
        });
    }
    Ok(())
}

/// Validate batch operation size
pub fn validate_batch_size(size: usize, field: &'static str) -> Result<(), ValidationError> {
    if size > MAX_BATCH_DELETE_SIZE {
        return Err(ValidationError::TooMany {
            field,
            max: MAX_BATCH_DELETE_SIZE,
        });
    }
    Ok(())
}

/// Validate device_id format (must be a valid UUID)
pub fn validate_device_id_format(device_id: &str) -> Result<(), ValidationError> {
    let trimmed = device_id.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::Empty { field: "device_id" });
    }
    // Validate UUID format
    if uuid::Uuid::parse_str(trimmed).is_err() {
        return Err(ValidationError::InvalidChars { field: "device_id" });
    }
    Ok(())
}

/// Status of host's dependent resources for deletion validation
#[derive(Debug, Clone, Copy, Default)]
pub struct HostDeletionStatus {
    /// Number of active sessions for this host
    pub session_count: usize,
    /// Number of archived sessions for this host
    pub archive_count: usize,
    /// Number of workspaces for this host (will be cascade deleted)
    pub workspace_count: usize,
}

/// Validate if a host can be safely deleted
///
/// Returns true if the host has no blocking dependencies (sessions, archives).
/// Workspaces are cascade deleted and don't block deletion.
pub fn validate_host_can_be_deleted(status: &HostDeletionStatus) -> bool {
    status.session_count == 0 && status.archive_count == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_device_name_valid() {
        assert!(validate_device_name("My iPhone").is_ok());
    }

    #[test]
    fn test_validate_device_name_empty() {
        assert!(validate_device_name("").is_err());
        assert!(validate_device_name("   ").is_err());
    }

    #[test]
    fn test_validate_device_name_too_long() {
        let name = "a".repeat(256);
        assert!(validate_device_name(&name).is_err());
    }

    #[test]
    fn test_validate_content_valid() {
        assert!(validate_content("Hello").is_ok());
    }

    #[test]
    fn test_validate_content_max_length() {
        let content = "a".repeat(100000);
        assert!(validate_content(&content).is_ok());
    }

    #[test]
    fn test_validate_content_too_long() {
        let content = "a".repeat(100001);
        assert!(validate_content(&content).is_err());
    }

    #[test]
    fn test_validate_batch_size_valid() {
        assert!(validate_batch_size(10, "archive_ids").is_ok());
    }

    #[test]
    fn test_validate_batch_size_at_max() {
        assert!(validate_batch_size(MAX_BATCH_DELETE_SIZE, "archive_ids").is_ok());
    }

    #[test]
    fn test_validate_batch_size_exceeds_max() {
        let result = validate_batch_size(MAX_BATCH_DELETE_SIZE + 1, "archive_ids");
        assert!(result.is_err());
        if let Err(ValidationError::TooMany { field, max }) = result {
            assert_eq!(field, "archive_ids");
            assert_eq!(max, MAX_BATCH_DELETE_SIZE);
        } else {
            panic!("Expected TooMany error");
        }
    }

    #[test]
    fn test_validate_batch_size_empty() {
        assert!(validate_batch_size(0, "archive_ids").is_ok());
    }

    #[test]
    fn test_validate_device_id_format_valid() {
        let uuid = uuid::Uuid::new_v4().to_string();
        assert!(validate_device_id_format(&uuid).is_ok());
    }

    #[test]
    fn test_validate_device_id_format_empty() {
        assert!(validate_device_id_format("").is_err());
        assert!(validate_device_id_format("   ").is_err());
    }

    #[test]
    fn test_validate_device_id_format_invalid() {
        assert!(validate_device_id_format("not-a-uuid").is_err());
        assert!(validate_device_id_format("12345").is_err());
    }

    #[test]
    fn test_validate_device_id_format_trims() {
        let uuid = uuid::Uuid::new_v4().to_string();
        assert!(validate_device_id_format(&format!("  {}  ", uuid)).is_ok());
    }

    #[test]
    fn test_validate_host_can_be_deleted_empty() {
        let status = HostDeletionStatus::default();
        assert!(validate_host_can_be_deleted(&status));
    }

    #[test]
    fn test_validate_host_cannot_be_deleted_with_sessions() {
        let status = HostDeletionStatus {
            session_count: 1,
            ..Default::default()
        };
        assert!(!validate_host_can_be_deleted(&status));
    }

    #[test]
    fn test_validate_host_cannot_be_deleted_with_archives() {
        let status = HostDeletionStatus {
            archive_count: 1,
            ..Default::default()
        };
        assert!(!validate_host_can_be_deleted(&status));
    }

    #[test]
    fn test_validate_host_can_be_deleted_with_workspaces() {
        let status = HostDeletionStatus {
            workspace_count: 10,
            ..Default::default()
        };
        assert!(validate_host_can_be_deleted(&status));
    }
}
