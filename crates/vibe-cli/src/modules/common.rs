use std::{
    env,
    path::{Path, PathBuf},
};

pub fn command_exists(command: &str) -> bool {
    let command = command.trim();
    if command.is_empty() {
        return false;
    }

    let path = Path::new(command);
    if path.components().count() > 1 || path.is_absolute() {
        return is_executable(path);
    }

    env::var_os("PATH")
        .map(|value| env::split_paths(&value).collect::<Vec<_>>())
        .into_iter()
        .flatten()
        .map(|dir| dir.join(command))
        .any(|candidate| is_executable(&candidate))
}

fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        metadata.permissions().mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    {
        let _ = metadata;
        true
    }
}

pub fn canonical_command_path(command: &str) -> Option<PathBuf> {
    let command = command.trim();
    if command.is_empty() {
        return None;
    }

    let path = Path::new(command);
    if (path.components().count() > 1 || path.is_absolute()) && is_executable(path) {
        return Some(path.to_path_buf());
    }

    env::var_os("PATH")
        .map(|value| env::split_paths(&value).collect::<Vec<_>>())
        .into_iter()
        .flatten()
        .map(|dir| dir.join(command))
        .find(|candidate| is_executable(candidate))
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::PermissionsExt};

    use tempfile::TempDir;

    use super::{canonical_command_path, command_exists};

    #[test]
    fn command_exists_accepts_absolute_executable_paths() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("demo-tool");
        fs::write(&path, "#!/usr/bin/env bash\nexit 0\n").unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();

        assert!(command_exists(path.to_str().unwrap()));
        assert_eq!(
            canonical_command_path(path.to_str().unwrap()).as_deref(),
            Some(path.as_path())
        );
    }
}
