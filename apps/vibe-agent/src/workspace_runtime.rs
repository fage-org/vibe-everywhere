use super::*;
use anyhow::{Context, Result, bail};
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

const WORKSPACE_PREVIEW_MAX_BYTES: u64 = 128 * 1024;
const WORKSPACE_PREVIEW_DEFAULT_LINE: u64 = 1;
const WORKSPACE_PREVIEW_DEFAULT_LIMIT: u64 = 200;
const WORKSPACE_PREVIEW_MAX_LIMIT: u64 = 500;

enum ClaimNextWorkspaceOutcome {
    Request(Option<WorkspaceOperationRequest>),
    DeviceMissing,
}

pub(crate) async fn workspace_loop(
    client: reqwest::Client,
    relay_url: String,
    profile: AgentProfile,
    auth: AgentAuthState,
    shared: SharedState,
    working_root: PathBuf,
    poll_interval_ms: u64,
) -> Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(poll_interval_ms));

    loop {
        interval.tick().await;

        match claim_next_workspace_request(&client, &relay_url, &profile.device_id, &auth).await {
            Ok(ClaimNextWorkspaceOutcome::Request(Some(request))) => {
                let workspace_client = client.clone();
                let workspace_relay_url = relay_url.clone();
                let workspace_device_id = profile.device_id.clone();
                let workspace_auth = auth.clone();
                let workspace_working_root = working_root.clone();
                tokio::spawn(async move {
                    let request_id = request.id().to_string();
                    if let Err(error) = run_workspace_request(
                        workspace_client,
                        workspace_relay_url,
                        workspace_device_id,
                        workspace_auth,
                        workspace_working_root,
                        request,
                    )
                    .await
                    {
                        eprintln!("workspace request {request_id} failed: {error:#}");
                    }
                });
            }
            Ok(ClaimNextWorkspaceOutcome::Request(None)) => {}
            Ok(ClaimNextWorkspaceOutcome::DeviceMissing) => {
                eprintln!(
                    "device {} missing on relay during workspace claim, re-registering",
                    profile.device_id
                );
                if let Err(error) =
                    register_current_device(&client, &relay_url, &profile, &shared, &auth).await
                {
                    eprintln!("device re-registration failed: {error:#}");
                }
            }
            Err(error) => {
                eprintln!("failed to claim workspace request: {error:#}");
            }
        }
    }
}

async fn claim_next_workspace_request(
    client: &reqwest::Client,
    relay_url: &str,
    device_id: &str,
    auth: &AgentAuthState,
) -> Result<ClaimNextWorkspaceOutcome> {
    let endpoint = format!("{relay_url}/api/devices/{device_id}/workspace/claim-next");
    let device_credential = auth.device_credential().await;
    let response = with_bearer(client.post(endpoint), device_credential.as_deref())
        .send()
        .await
        .context("failed to claim workspace request")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(ClaimNextWorkspaceOutcome::DeviceMissing);
    }

    let response = response
        .error_for_status()
        .context("relay rejected workspace request claim")?
        .json::<ClaimWorkspaceOperationResponse>()
        .await
        .context("failed to decode claim workspace response")?;

    Ok(ClaimNextWorkspaceOutcome::Request(response.request))
}

async fn run_workspace_request(
    client: reqwest::Client,
    relay_url: String,
    device_id: String,
    auth: AgentAuthState,
    working_root: PathBuf,
    request: WorkspaceOperationRequest,
) -> Result<()> {
    let request_id = request.id().to_string();
    let result = match &request {
        WorkspaceOperationRequest::Browse {
            device_id,
            session_cwd,
            path,
            ..
        } => handle_workspace_browse(
            device_id,
            &working_root,
            session_cwd.as_deref(),
            path.as_deref(),
        )
        .map(|response| WorkspaceOperationResult::Browse { response }),
        WorkspaceOperationRequest::Preview {
            device_id,
            session_cwd,
            path,
            line,
            limit,
            ..
        } => handle_workspace_preview(
            device_id,
            &working_root,
            session_cwd.as_deref(),
            path,
            *line,
            *limit,
        )
        .map(|response| WorkspaceOperationResult::Preview { response }),
    }
    .unwrap_or_else(|error| WorkspaceOperationResult::Error {
        message: error.to_string(),
    });

    complete_workspace_request(&client, &relay_url, &request_id, &device_id, result, &auth).await
}

fn handle_workspace_browse(
    device_id: &str,
    working_root: &Path,
    session_cwd: Option<&str>,
    requested_path: Option<&str>,
) -> Result<WorkspaceBrowseResponse> {
    let root_path = resolve_workspace_root(working_root, session_cwd)?;
    let path = resolve_existing_workspace_path(&root_path, requested_path)?;

    if !path.is_dir() {
        bail!("workspace path is not a directory: {}", path.display());
    }

    let mut entries = fs::read_dir(&path)
        .with_context(|| format!("failed to read directory {}", path.display()))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| build_workspace_entry(&entry).ok())
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| match (&left.kind, &right.kind) {
        (WorkspaceEntryKind::Directory, WorkspaceEntryKind::File) => std::cmp::Ordering::Less,
        (WorkspaceEntryKind::File, WorkspaceEntryKind::Directory) => std::cmp::Ordering::Greater,
        _ => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
    });

    Ok(WorkspaceBrowseResponse {
        device_id: device_id.to_string(),
        root_path: root_path.display().to_string(),
        path: path.display().to_string(),
        parent_path: workspace_parent_path(&root_path, &path),
        entries,
    })
}

fn handle_workspace_preview(
    device_id: &str,
    working_root: &Path,
    session_cwd: Option<&str>,
    requested_path: &str,
    requested_line: Option<u64>,
    requested_limit: Option<u64>,
) -> Result<WorkspaceFilePreviewResponse> {
    let root_path = resolve_workspace_root(working_root, session_cwd)?;
    let path = resolve_existing_workspace_path(&root_path, Some(requested_path))?;
    let metadata =
        fs::metadata(&path).with_context(|| format!("failed to read {}", path.display()))?;

    if metadata.is_dir() {
        return Ok(WorkspaceFilePreviewResponse {
            device_id: device_id.to_string(),
            root_path: root_path.display().to_string(),
            path: path.display().to_string(),
            kind: WorkspacePreviewKind::Directory,
            content: None,
            truncated: false,
            line: None,
            total_lines: None,
            size_bytes: metadata.len(),
        });
    }

    let size_bytes = metadata.len();
    let file = File::open(&path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(WORKSPACE_PREVIEW_MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .with_context(|| format!("failed to read {}", path.display()))?;

    let mut truncated = size_bytes > WORKSPACE_PREVIEW_MAX_BYTES;
    if bytes.len() as u64 > WORKSPACE_PREVIEW_MAX_BYTES {
        bytes.truncate(WORKSPACE_PREVIEW_MAX_BYTES as usize);
        truncated = true;
    }

    if bytes.contains(&0) {
        return Ok(WorkspaceFilePreviewResponse {
            device_id: device_id.to_string(),
            root_path: root_path.display().to_string(),
            path: path.display().to_string(),
            kind: WorkspacePreviewKind::Binary,
            content: None,
            truncated,
            line: None,
            total_lines: None,
            size_bytes,
        });
    }

    let text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(_) => {
            return Ok(WorkspaceFilePreviewResponse {
                device_id: device_id.to_string(),
                root_path: root_path.display().to_string(),
                path: path.display().to_string(),
                kind: WorkspacePreviewKind::Binary,
                content: None,
                truncated,
                line: None,
                total_lines: None,
                size_bytes,
            });
        }
    };

    let line = requested_line
        .unwrap_or(WORKSPACE_PREVIEW_DEFAULT_LINE)
        .max(1);
    let limit = requested_limit
        .unwrap_or(WORKSPACE_PREVIEW_DEFAULT_LIMIT)
        .clamp(1, WORKSPACE_PREVIEW_MAX_LIMIT) as usize;
    let total_lines = text.lines().count() as u64;
    let content = slice_text_by_lines(&text, line as usize, Some(limit));
    if (line.saturating_sub(1) + limit as u64) < total_lines {
        truncated = true;
    }

    Ok(WorkspaceFilePreviewResponse {
        device_id: device_id.to_string(),
        root_path: root_path.display().to_string(),
        path: path.display().to_string(),
        kind: WorkspacePreviewKind::Text,
        content: Some(content),
        truncated,
        line: Some(line),
        total_lines: Some(total_lines),
        size_bytes,
    })
}

async fn complete_workspace_request(
    client: &reqwest::Client,
    relay_url: &str,
    request_id: &str,
    device_id: &str,
    result: WorkspaceOperationResult,
    auth: &AgentAuthState,
) -> Result<()> {
    let endpoint = format!("{relay_url}/api/workspace/requests/{request_id}/complete");
    let device_credential = auth.device_credential().await;
    with_bearer(client.post(endpoint), device_credential.as_deref())
        .json(&CompleteWorkspaceOperationRequest {
            device_id: device_id.to_string(),
            result,
        })
        .send()
        .await
        .context("failed to complete workspace request")?
        .error_for_status()
        .context("relay rejected workspace completion")?;

    Ok(())
}

fn resolve_workspace_root(working_root: &Path, session_cwd: Option<&str>) -> Result<PathBuf> {
    let root = resolve_task_cwd(working_root, session_cwd);
    ensure_task_cwd(&root)
}

fn resolve_existing_workspace_path(root: &Path, requested_path: Option<&str>) -> Result<PathBuf> {
    let path = match requested_path {
        Some(value) if !value.trim().is_empty() => {
            let candidate = PathBuf::from(value);
            if candidate.is_absolute() {
                candidate
            } else {
                root.join(candidate)
            }
        }
        _ => root.to_path_buf(),
    };
    let path = path
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", path.display()))?;
    ensure_workspace_path_within_root(root, &path)?;
    Ok(path)
}

fn ensure_workspace_path_within_root(root: &Path, path: &Path) -> Result<()> {
    if path.starts_with(root) {
        Ok(())
    } else {
        bail!(
            "workspace path {} is outside workspace root {}",
            path.display(),
            root.display()
        )
    }
}

fn workspace_parent_path(root: &Path, path: &Path) -> Option<String> {
    if path == root {
        return None;
    }

    path.parent()
        .filter(|parent| parent.starts_with(root))
        .map(|parent| parent.display().to_string())
}

fn build_workspace_entry(entry: &fs::DirEntry) -> Result<WorkspaceEntry> {
    let path = entry
        .path()
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", entry.path().display()))?;
    let metadata = entry
        .metadata()
        .with_context(|| format!("failed to read {}", path.display()))?;
    let modified_at_epoch_ms = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64);

    Ok(WorkspaceEntry {
        path: path.display().to_string(),
        name: entry.file_name().to_string_lossy().to_string(),
        kind: if metadata.is_dir() {
            WorkspaceEntryKind::Directory
        } else {
            WorkspaceEntryKind::File
        },
        size_bytes: metadata.is_file().then_some(metadata.len()),
        modified_at_epoch_ms,
    })
}

fn slice_text_by_lines(content: &str, line: usize, limit: Option<usize>) -> String {
    let start = line.saturating_sub(1);
    let lines = content.lines().collect::<Vec<_>>();
    if start >= lines.len() {
        return String::new();
    }

    let end = limit
        .map(|value| start.saturating_add(value).min(lines.len()))
        .unwrap_or(lines.len());
    lines[start..end].join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn temp_workspace_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("vibe-workspace-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn resolve_existing_workspace_path_rejects_escape() {
        let root = temp_workspace_dir();
        let sibling = root
            .parent()
            .unwrap()
            .join(format!("escape-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&sibling).unwrap();

        let result = resolve_existing_workspace_path(&root, Some(sibling.to_str().unwrap()));
        assert!(result.is_err());

        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&sibling);
    }

    #[test]
    fn workspace_browse_and_preview_respect_workspace_root() {
        let root = temp_workspace_dir();
        let nested = root.join("src");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(root.join("README.md"), "line 1\nline 2\nline 3\n").unwrap();

        let browse = handle_workspace_browse("device-1", &root, None, None).unwrap();
        assert_eq!(browse.root_path, root.display().to_string());
        assert_eq!(browse.entries.len(), 2);
        assert_eq!(browse.entries[0].kind, WorkspaceEntryKind::Directory);
        assert_eq!(browse.entries[0].name, "src");

        let preview = handle_workspace_preview(
            "device-1",
            &root,
            None,
            root.join("README.md").to_str().unwrap(),
            Some(2),
            Some(1),
        )
        .unwrap();
        assert_eq!(preview.kind, WorkspacePreviewKind::Text);
        assert_eq!(preview.content.as_deref(), Some("line 2"));
        assert_eq!(preview.total_lines, Some(3));

        let _ = std::fs::remove_dir_all(&root);
    }
}
