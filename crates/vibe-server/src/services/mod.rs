//! Service modules for external integrations

pub mod token_validation;

pub use token_validation::{TokenValidationError, validate_vendor_token};
