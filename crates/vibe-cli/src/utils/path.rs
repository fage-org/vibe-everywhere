use anyhow::{Result, bail};

pub fn ensure_non_empty_path(path: &str) -> Result<&str> {
    if path.trim().is_empty() {
        bail!("path is required");
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::ensure_non_empty_path;

    #[test]
    fn path_validation_rejects_empty_values() {
        assert!(ensure_non_empty_path(" ").is_err());
    }
}
