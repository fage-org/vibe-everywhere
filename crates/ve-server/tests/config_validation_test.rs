//! Tests for configuration validation
//!
//! Tests for JWT secret validation and other security-related config checks.

/// JWT secret validation error
#[derive(Debug, Clone, PartialEq)]
enum JwtSecretError {
    TooShort { length: usize, minimum: usize },
    PlaceholderValue(String),
}

/// Validate JWT secret meets security requirements
fn validate_jwt_secret(secret: &str) -> Result<(), JwtSecretError> {
    const MIN_LENGTH: usize = 32;

    // Check minimum length
    if secret.len() < MIN_LENGTH {
        return Err(JwtSecretError::TooShort {
            length: secret.len(),
            minimum: MIN_LENGTH,
        });
    }

    // Check for placeholder values
    let lower = secret.to_lowercase();
    let placeholders = [
        "your-secret-key",
        "your_secret_key",
        "secret",
        "jwt_secret",
        "change_me",
        "changeme",
        "example",
        "placeholder",
        "default",
        "test",
        "dev",
        "development",
    ];

    for placeholder in &placeholders {
        if lower.contains(placeholder) {
            return Err(JwtSecretError::PlaceholderValue(secret.to_string()));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_jwt_secret_passes_validation() {
        // A strong, random secret
        let secret = "aB3dE7fG9hJ2kL5mN8pQ1rS4tU6vW0xY";
        assert!(validate_jwt_secret(secret).is_ok());
    }

    #[test]
    fn jwt_secret_too_short_fails() {
        let secret = "too_short";
        let result = validate_jwt_secret(secret);
        assert!(result.is_err());
        assert_eq!(
            result,
            Err(JwtSecretError::TooShort {
                length: 9,
                minimum: 32
            })
        );
    }

    #[test]
    fn jwt_secret_exactly_32_chars_passes() {
        let secret = "12345678901234567890123456789012"; // exactly 32 chars
        assert!(validate_jwt_secret(secret).is_ok());
    }

    #[test]
    fn jwt_secret_31_chars_fails() {
        let secret = "1234567890123456789012345678901"; // 31 chars
        let result = validate_jwt_secret(secret);
        assert!(result.is_err());
        assert_eq!(
            result,
            Err(JwtSecretError::TooShort {
                length: 31,
                minimum: 32
            })
        );
    }

    #[test]
    fn jwt_secret_with_placeholder_your_secret_key_fails() {
        let secret = "your-secret-key-12345678901234567890"; // 38 chars but contains placeholder
        let result = validate_jwt_secret(secret);
        assert!(result.is_err());
        assert!(matches!(result, Err(JwtSecretError::PlaceholderValue(_))));
    }

    #[test]
    fn jwt_secret_with_placeholder_secret_fails() {
        let secret = "my_super_secret_key_12345678901234567890"; // contains "secret"
        let result = validate_jwt_secret(secret);
        assert!(result.is_err());
        assert!(matches!(result, Err(JwtSecretError::PlaceholderValue(_))));
    }

    #[test]
    fn jwt_secret_with_placeholder_change_me_fails() {
        let secret = "change_me_to_something_secure_1234567890"; // contains "change_me"
        let result = validate_jwt_secret(secret);
        assert!(result.is_err());
        assert!(matches!(result, Err(JwtSecretError::PlaceholderValue(_))));
    }

    #[test]
    fn jwt_secret_with_placeholder_example_fails() {
        let secret = "example_jwt_secret_key_1234567890123456"; // contains "example"
        let result = validate_jwt_secret(secret);
        assert!(result.is_err());
        assert!(matches!(result, Err(JwtSecretError::PlaceholderValue(_))));
    }

    #[test]
    fn jwt_secret_with_placeholder_default_fails() {
        let secret = "default_jwt_secret_key_1234567890123456"; // contains "default"
        let result = validate_jwt_secret(secret);
        assert!(result.is_err());
        assert!(matches!(result, Err(JwtSecretError::PlaceholderValue(_))));
    }

    #[test]
    fn jwt_secret_case_insensitive_placeholder_detection() {
        let secret = "YOUR-SECRET-KEY-12345678901234567890"; // uppercase placeholder
        let result = validate_jwt_secret(secret);
        assert!(result.is_err());
        assert!(matches!(result, Err(JwtSecretError::PlaceholderValue(_))));
    }

    #[test]
    fn jwt_secret_empty_fails() {
        let secret = "";
        let result = validate_jwt_secret(secret);
        assert!(result.is_err());
        assert_eq!(
            result,
            Err(JwtSecretError::TooShort {
                length: 0,
                minimum: 32
            })
        );
    }

    #[test]
    fn jwt_secret_with_special_chars_passes() {
        let secret = "p@ssw0rd!#$%^&*()_+-=[]{}|;':\",./<>?"; // 32 chars with special chars
        assert!(validate_jwt_secret(secret).is_ok());
    }

    #[test]
    fn jwt_secret_long_random_string_passes() {
        let secret = "kJ8mN2pQ5rS9tU1vW4xY7zA3bC6dE0fGhIjKlMnOpQrStUvWxYz";
        assert!(validate_jwt_secret(secret).is_ok());
    }
}
