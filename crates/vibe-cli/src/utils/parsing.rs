pub fn normalize_provider_name(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::normalize_provider_name;

    #[test]
    fn provider_name_normalization_is_stable() {
        assert_eq!(normalize_provider_name(" Claude "), "claude");
    }
}
