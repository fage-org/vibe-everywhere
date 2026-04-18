//! Input Validation Tests
//!
//! Tests for input field length validation.

use ve_server::validation::{
    validate_device_name, validate_host_name, validate_title, validate_content,
    ValidationError, MAX_DEVICE_NAME_LENGTH, MAX_HOST_NAME_LENGTH,
    MAX_TITLE_LENGTH, MAX_CONTENT_LENGTH,
};

#[test]
fn validate_device_name_accepts_valid() {
    assert!(validate_device_name("My iPhone").is_ok());
}

#[test]
fn validate_device_name_accepts_max_length() {
    let name = "a".repeat(MAX_DEVICE_NAME_LENGTH);
    assert!(validate_device_name(&name).is_ok());
}

#[test]
fn validate_device_name_rejects_too_long() {
    let name = "a".repeat(MAX_DEVICE_NAME_LENGTH + 1);
    let result = validate_device_name(&name);
    assert!(result.is_err());
    if let Err(ValidationError::TooLong { field, max }) = result {
        assert_eq!(field, "device_name");
        assert_eq!(max, MAX_DEVICE_NAME_LENGTH);
    } else {
        panic!("Expected TooLong error");
    }
}

#[test]
fn validate_device_name_rejects_empty() {
    let result = validate_device_name("");
    assert!(result.is_err());
    if let Err(ValidationError::Empty { field }) = result {
        assert_eq!(field, "device_name");
    } else {
        panic!("Expected Empty error");
    }
}

#[test]
fn validate_host_name_accepts_valid() {
    assert!(validate_host_name("My MacBook Pro").is_ok());
}

#[test]
fn validate_host_name_rejects_too_long() {
    let name = "a".repeat(MAX_HOST_NAME_LENGTH + 1);
    assert!(validate_host_name(&name).is_err());
}

#[test]
fn validate_host_name_rejects_empty() {
    assert!(validate_host_name("").is_err());
}

#[test]
fn validate_title_accepts_valid() {
    assert!(validate_title("Fix authentication bug").is_ok());
}

#[test]
fn validate_title_rejects_too_long() {
    let title = "a".repeat(MAX_TITLE_LENGTH + 1);
    assert!(validate_title(&title).is_err());
}

#[test]
fn validate_title_rejects_empty() {
    assert!(validate_title("").is_err());
}

#[test]
fn validate_content_accepts_valid() {
    assert!(validate_content("Hello, how are you?").is_ok());
}

#[test]
fn validate_content_accepts_max_length() {
    let content = "a".repeat(MAX_CONTENT_LENGTH);
    assert!(validate_content(&content).is_ok());
}

#[test]
fn validate_content_rejects_too_long() {
    let content = "a".repeat(MAX_CONTENT_LENGTH + 1);
    assert!(validate_content(&content).is_err());
}

#[test]
fn validate_content_rejects_empty() {
    assert!(validate_content("").is_err());
}

#[test]
fn validate_trims_whitespace() {
    // Name with only whitespace should be treated as empty
    assert!(validate_device_name("   ").is_err());
}

#[test]
fn validate_accepts_whitespace_padded_valid() {
    // Whitespace-padded valid content should still work after trim
    assert!(validate_device_name("  valid name  ").is_ok());
}

#[test]
fn validation_error_display_empty() {
    let err = ValidationError::Empty { field: "test_field" };
    let msg = format!("{}", err);
    assert!(msg.contains("test_field"));
    assert!(msg.contains("empty"));
}

#[test]
fn validation_error_display_too_long() {
    let err = ValidationError::TooLong { field: "test_field", max: 100 };
    let msg = format!("{}", err);
    assert!(msg.contains("test_field"));
    assert!(msg.contains("100"));
}
