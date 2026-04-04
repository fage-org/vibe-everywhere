use serde::Serialize;

pub fn print_heading(title: &str) -> String {
    format!("## {title}")
}

pub fn format_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "null".into())
}

#[cfg(test)]
mod tests {
    use super::{format_json, print_heading};

    #[test]
    fn heading_is_stable() {
        assert_eq!(print_heading("Daemon"), "## Daemon");
    }

    #[test]
    fn json_is_pretty() {
        let output = format_json(&serde_json::json!({"ok": true}));
        assert!(output.contains('\n'));
    }
}
