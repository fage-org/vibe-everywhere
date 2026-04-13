//! Service modules for external integrations

pub mod token_validation;

pub use token_validation::{validate_vendor_token, TokenValidationError};