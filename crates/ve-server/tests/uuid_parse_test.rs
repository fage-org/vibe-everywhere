//! Tests for UUID parsing helpers
//!
//! Tests for safe UUID parsing with proper error handling instead of unwrap().

use std::fmt;
use uuid::Uuid;

/// Error type for UUID parsing failures
#[derive(Debug, Clone, PartialEq)]
pub struct UuidParseError {
    pub input: String,
    pub context: String,
}

impl fmt::Display for UuidParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse UUID '{}': {}", self.input, self.context)
    }
}

impl std::error::Error for UuidParseError {}

/// Parse a UUID string with context for better error messages
pub fn parse_uuid(input: &str, context: &str) -> Result<Uuid, UuidParseError> {
    Uuid::parse_str(input).map_err(|e| UuidParseError {
        input: input.to_string(),
        context: format!("{}: {}", context, e),
    })
}

/// Parse a UUID string with a field name context
pub fn parse_uuid_field(input: &str, field_name: &str) -> Result<Uuid, UuidParseError> {
    parse_uuid(input, &format!("invalid {}", field_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_uuid_succeeds() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let result = parse_uuid(uuid_str, "test context");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().to_string(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[test]
    fn parse_valid_uuid_no_hyphens_succeeds() {
        let uuid_str = "550e8400e29b41d4a716446655440000";
        let result = parse_uuid(uuid_str, "test context");
        assert!(result.is_ok());
    }

    #[test]
    fn parse_valid_uuid_uppercase_succeeds() {
        let uuid_str = "550E8400-E29B-41D4-A716-446655440000";
        let result = parse_uuid(uuid_str, "test context");
        assert!(result.is_ok());
    }

    #[test]
    fn parse_invalid_uuid_returns_error() {
        let invalid_uuid = "not-a-uuid";
        let result = parse_uuid(invalid_uuid, "test context");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.input, "not-a-uuid");
        assert!(error.context.contains("test context"));
    }

    #[test]
    fn parse_empty_uuid_returns_error() {
        let result = parse_uuid("", "empty uuid test");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.input, "");
    }

    #[test]
    fn parse_uuid_with_wrong_format_returns_error() {
        let wrong_format = "550e8400-e29b-41d4-a716"; // too short
        let result = parse_uuid(wrong_format, "incomplete uuid");
        assert!(result.is_err());
    }

    #[test]
    fn parse_uuid_with_invalid_chars_returns_error() {
        let invalid_chars = "550e8400-e29b-41d4-a716-44665544000g"; // 'g' is invalid hex
        let result = parse_uuid(invalid_chars, "invalid chars");
        assert!(result.is_err());
    }

    #[test]
    fn parse_uuid_field_provides_field_context() {
        let invalid_uuid = "invalid";
        let result = parse_uuid_field(invalid_uuid, "session_id");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.context.contains("session_id"));
    }

    #[test]
    fn parse_uuid_field_valid_succeeds() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let result = parse_uuid_field(uuid_str, "host_id");
        assert!(result.is_ok());
    }

    #[test]
    fn uuid_parse_error_display() {
        let error = UuidParseError {
            input: "bad-uuid".to_string(),
            context: "invalid format".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("bad-uuid"));
        assert!(display.contains("invalid format"));
    }

    #[test]
    fn parse_uuid_trimmed_input() {
        // Note: uuid crate doesn't auto-trim, so this should fail
        let uuid_with_spaces = " 550e8400-e29b-41d4-a716-446655440000 ";
        let result = parse_uuid(uuid_with_spaces, "spaced uuid");
        assert!(result.is_err());
    }

    #[test]
    fn parse_uuid_nil_succeeds() {
        let nil_uuid = "00000000-0000-0000-0000-000000000000";
        let result = parse_uuid(nil_uuid, "nil uuid");
        assert!(result.is_ok());
        assert!(result.unwrap().is_nil());
    }
}
