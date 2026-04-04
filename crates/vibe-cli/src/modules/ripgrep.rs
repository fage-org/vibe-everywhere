use std::process::Command;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use crate::modules::common::command_exists;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchHit {
    pub path: String,
    pub line_number: u64,
    pub text: String,
}

pub fn available() -> bool {
    command_exists("rg")
}

pub fn assert_available() -> Result<()> {
    if !available() {
        bail!("ripgrep is not available");
    }
    Ok(())
}

pub fn search(pattern: &str, root: &str) -> Result<Vec<SearchHit>> {
    assert_available()?;
    let output = Command::new("rg")
        .args(["-n", "--color", "never", pattern, root])
        .output()?;
    if output.status.code() == Some(1) {
        return Ok(Vec::new());
    }
    if !output.status.success() {
        bail!(
            "ripgrep search failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(3, ':');
            let path = parts.next()?.to_string();
            let line_number = parts.next()?.parse().ok()?;
            let text = parts.next().unwrap_or_default().to_string();
            Some(SearchHit {
                path,
                line_number,
                text,
            })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::{available, search};

    #[test]
    fn searches_files_when_ripgrep_is_available() {
        if !available() {
            return;
        }
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("demo.txt");
        fs::write(&file, "alpha\nbeta target\ngamma\n").unwrap();

        let hits = search("target", temp_dir.path().to_str().unwrap()).unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].path.ends_with("demo.txt"));
        assert_eq!(hits[0].line_number, 2);
        assert_eq!(hits[0].text, "beta target");
    }
}
