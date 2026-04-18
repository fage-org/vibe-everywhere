//! Tests for device validation in settings API

use ve_server::validation::validate_device_id_format;

/// Test that validate_device_id_format returns error for empty string
#[test]
fn validate_device_id_format_rejects_empty() {
    let result = validate_device_id_format("");
    assert!(result.is_err());
}

/// Test that validate_device_id_format accepts valid UUID format
#[test]
fn validate_device_id_format_accepts_uuid_format() {
    let valid_uuid = uuid::Uuid::new_v4().to_string();
    let result = validate_device_id_format(&valid_uuid);
    assert!(result.is_ok());
}

/// Test that validate_device_id_format rejects invalid UUID format
#[test]
fn validate_device_id_format_rejects_invalid_format() {
    let result = validate_device_id_format("not-a-uuid");
    assert!(result.is_err());
}

/// Test that validate_device_id_format trims whitespace
#[test]
fn validate_device_id_format_trims_whitespace() {
    let valid_uuid = uuid::Uuid::new_v4().to_string();
    let result = validate_device_id_format(&format!("  {}  ", valid_uuid));
    assert!(result.is_ok());
}
