use std::{
    io,
    path::Path,
    time::{Duration, SystemTime},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatcherMode {
    Poll,
}

pub fn default_mode() -> WatcherMode {
    WatcherMode::Poll
}

pub fn modified_at(path: &Path) -> io::Result<SystemTime> {
    std::fs::metadata(path)?.modified()
}

pub async fn wait_for_change(
    path: &Path,
    baseline: SystemTime,
    timeout: Duration,
    poll_interval: Duration,
) -> io::Result<bool> {
    let started = std::time::Instant::now();
    while started.elapsed() < timeout {
        if modified_at(path)? > baseline {
            return Ok(true);
        }
        tokio::time::sleep(poll_interval).await;
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use std::{fs, time::Duration};

    use tempfile::TempDir;

    use super::{modified_at, wait_for_change};

    #[tokio::test]
    async fn detects_file_changes_by_polling() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("watch.txt");
        fs::write(&path, "alpha").unwrap();
        let baseline = modified_at(&path).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        fs::write(&path, "beta").unwrap();

        let changed = wait_for_change(
            &path,
            baseline,
            Duration::from_secs(1),
            Duration::from_millis(25),
        )
        .await
        .unwrap();
        assert!(changed);
    }
}
