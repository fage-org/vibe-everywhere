use std::process::Command;

use anyhow::{Result, bail};

use crate::modules::common::command_exists;

pub fn available() -> bool {
    command_exists("difft")
}

pub fn render_diff(left: &str, right: &str) -> Result<String> {
    let output = if available() {
        Command::new("difft")
            .args(["--color", "never", left, right])
            .output()?
    } else {
        Command::new("diff").args(["-u", left, right]).output()?
    };
    if !output.status.success() && output.status.code() != Some(1) {
        bail!(
            "diff rendering failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if stdout.trim().is_empty() {
        Ok(String::from_utf8_lossy(&output.stderr).to_string())
    } else {
        Ok(stdout)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::render_diff;

    #[test]
    fn renders_diff_between_two_files() {
        let temp_dir = TempDir::new().unwrap();
        let left = temp_dir.path().join("left.txt");
        let right = temp_dir.path().join("right.txt");
        fs::write(&left, "alpha\nbeta\n").unwrap();
        fs::write(&right, "alpha\ngamma\n").unwrap();

        let rendered = render_diff(left.to_str().unwrap(), right.to_str().unwrap()).unwrap();
        assert!(rendered.contains("beta") || rendered.contains("gamma"));
    }
}
