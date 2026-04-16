//! Input Validation
//!
//! Provides validation helpers for user input fields.

use thiserror::Error;

/// Maximum lengths for various fields
pub const MAX_DEVICE_NAME_LENGTH: usize = 255;
pub const MAX_HOST_NAME_LENGTH: usize = 255;
pub const MAX_TITLE_LENGTH: usize = 500;
pub const MAX_CONTENT_LENGTH: usize = 100000; // 100KB

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
}
