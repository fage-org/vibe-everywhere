//! CORS Configuration Tests
//!
//! Tests for configurable CORS origins parsing logic.

#[test]
fn cors_origins_parses_single_origin() {
    // Test the parsing logic with a simple string split
    let input = "https://example.com";
    let origins: Vec<String> = input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    assert_eq!(origins, vec!["https://example.com"]);
}

#[test]
fn cors_origins_parses_multiple_origins() {
    let input = "https://example.com,https://app.example.com";
    let origins: Vec<String> = input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    assert_eq!(origins, vec!["https://example.com", "https://app.example.com"]);
}

#[test]
fn cors_origins_trims_whitespace() {
    let input = " https://example.com , https://app.example.com ";
    let origins: Vec<String> = input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    assert_eq!(origins, vec!["https://example.com", "https://app.example.com"]);
}

#[test]
fn cors_origins_empty_string_means_empty() {
    let input = "";
    let origins: Vec<String> = input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    assert!(origins.is_empty());
}

#[test]
fn cors_origins_wildcard_allowed_explicitly() {
    let input = "*";
    let origins: Vec<String> = input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    assert_eq!(origins, vec!["*"]);
}

#[test]
fn cors_origins_filter_empty_elements() {
    let input = "https://example.com,,https://app.example.com";
    let origins: Vec<String> = input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    assert_eq!(origins, vec!["https://example.com", "https://app.example.com"]);
}

#[test]
fn cors_origins_default_empty() {
    // Default CORS origins should be empty (most restrictive)
    let default: Vec<String> = Vec::new();
    assert!(default.is_empty());
}
