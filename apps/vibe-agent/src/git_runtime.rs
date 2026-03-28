use super::*;
use anyhow::{Context, Result, bail};
#[cfg(test)]
use std::fs;
use std::{
    path::{Path, PathBuf},
    process::{Command as StdCommand, Output},
};

const GIT_RECENT_COMMITS_LIMIT: usize = 5;

enum ClaimNextGitOutcome {
    Request(Option<GitOperationRequest>),
    DeviceMissing,
}

#[derive(Default)]
struct ParsedBranchState {
    branch_name: Option<String>,
    upstream_branch: Option<String>,
    ahead_count: u64,
    behind_count: u64,
}

pub(crate) async fn git_loop(
    client: reqwest::Client,
    relay_url: String,
    profile: AgentProfile,
    shared: SharedState,
    working_root: PathBuf,
    poll_interval_ms: u64,
) -> Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(poll_interval_ms));

    loop {
        interval.tick().await;

        match claim_next_git_request(&client, &relay_url, &profile.device_id).await {
            Ok(ClaimNextGitOutcome::Request(Some(request))) => {
                let git_client = client.clone();
                let git_relay_url = relay_url.clone();
                let git_device_id = profile.device_id.clone();
                let git_working_root = working_root.clone();
                tokio::spawn(async move {
                    let request_id = request.id().to_string();
                    if let Err(error) = run_git_request(
                        git_client,
                        git_relay_url,
                        git_device_id,
                        git_working_root,
                        request,
                    )
                    .await
                    {
                        eprintln!("git request {request_id} failed: {error:#}");
                    }
                });
            }
            Ok(ClaimNextGitOutcome::Request(None)) => {}
            Ok(ClaimNextGitOutcome::DeviceMissing) => {
                eprintln!(
                    "device {} missing on relay during git claim, re-registering",
                    profile.device_id
                );
                if let Err(error) =
                    register_current_device(&client, &relay_url, &profile, &shared).await
                {
                    eprintln!("device re-registration failed: {error:#}");
                }
            }
            Err(error) => {
                eprintln!("failed to claim git request: {error:#}");
            }
        }
    }
}

async fn claim_next_git_request(
    client: &reqwest::Client,
    relay_url: &str,
    device_id: &str,
) -> Result<ClaimNextGitOutcome> {
    let endpoint = format!("{relay_url}/api/devices/{device_id}/git/claim-next");
    let response = client
        .post(endpoint)
        .send()
        .await
        .context("failed to claim git request")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(ClaimNextGitOutcome::DeviceMissing);
    }

    let response = response
        .error_for_status()
        .context("relay rejected git request claim")?
        .json::<ClaimGitOperationResponse>()
        .await
        .context("failed to decode claim git response")?;

    Ok(ClaimNextGitOutcome::Request(response.request))
}

async fn run_git_request(
    client: reqwest::Client,
    relay_url: String,
    device_id: String,
    working_root: PathBuf,
    request: GitOperationRequest,
) -> Result<()> {
    let request_id = request.id().to_string();
    let result = match &request {
        GitOperationRequest::Inspect {
            device_id,
            session_cwd,
            ..
        } => handle_git_inspect(device_id, &working_root, session_cwd.as_deref())
            .map(|response| GitOperationResult::Inspect { response }),
    }
    .unwrap_or_else(|error| GitOperationResult::Error {
        message: error.to_string(),
    });

    complete_git_request(&client, &relay_url, &request_id, &device_id, result).await
}

fn handle_git_inspect(
    device_id: &str,
    working_root: &Path,
    session_cwd: Option<&str>,
) -> Result<GitInspectResponse> {
    let workspace_root = resolve_git_workspace_root(working_root, session_cwd)?;
    let workspace_root_display = workspace_root.display().to_string();

    if which::which("git").is_err() {
        return Ok(GitInspectResponse {
            device_id: device_id.to_string(),
            workspace_root: workspace_root_display,
            repo_root: None,
            scope_path: None,
            state: GitInspectState::GitUnavailable,
            branch_name: None,
            upstream_branch: None,
            ahead_count: 0,
            behind_count: 0,
            has_commits: false,
            changed_files: vec![],
            recent_commits: vec![],
            diff_stats: GitDiffStats::default(),
        });
    }

    let Some(repo_root) = resolve_repo_root(&workspace_root)? else {
        return Ok(GitInspectResponse {
            device_id: device_id.to_string(),
            workspace_root: workspace_root_display,
            repo_root: None,
            scope_path: None,
            state: GitInspectState::NotRepository,
            branch_name: None,
            upstream_branch: None,
            ahead_count: 0,
            behind_count: 0,
            has_commits: false,
            changed_files: vec![],
            recent_commits: vec![],
            diff_stats: GitDiffStats::default(),
        });
    };

    let scope_path = workspace_scope_path(&repo_root, &workspace_root)?;
    let status_output = run_git(
        &repo_root,
        git_args_with_scope(
            &["status", "--porcelain=v1", "--branch", "--no-renames", "-z"],
            scope_path.as_deref(),
        ),
    )?;
    ensure_git_success(&status_output, "status")?;

    let (branch_state, changed_files, mut diff_stats) =
        parse_git_status(&status_output.stdout, &repo_root)?;

    populate_numstat(
        &repo_root,
        git_args_with_scope(&["diff", "--numstat", "--cached"], scope_path.as_deref()),
        &mut diff_stats.staged_additions,
        &mut diff_stats.staged_deletions,
    )?;
    populate_numstat(
        &repo_root,
        git_args_with_scope(&["diff", "--numstat"], scope_path.as_deref()),
        &mut diff_stats.unstaged_additions,
        &mut diff_stats.unstaged_deletions,
    )?;

    let has_commits = git_head_exists(&repo_root)?;
    let recent_commits = if has_commits {
        load_recent_commits(&repo_root)?
    } else {
        vec![]
    };

    Ok(GitInspectResponse {
        device_id: device_id.to_string(),
        workspace_root: workspace_root_display,
        repo_root: Some(repo_root.display().to_string()),
        scope_path: Some(scope_path.unwrap_or_else(|| ".".to_string())),
        state: GitInspectState::Ready,
        branch_name: branch_state.branch_name,
        upstream_branch: branch_state.upstream_branch,
        ahead_count: branch_state.ahead_count,
        behind_count: branch_state.behind_count,
        has_commits,
        changed_files,
        recent_commits,
        diff_stats,
    })
}

async fn complete_git_request(
    client: &reqwest::Client,
    relay_url: &str,
    request_id: &str,
    device_id: &str,
    result: GitOperationResult,
) -> Result<()> {
    let endpoint = format!("{relay_url}/api/git/requests/{request_id}/complete");
    client
        .post(endpoint)
        .json(&CompleteGitOperationRequest {
            device_id: device_id.to_string(),
            result,
        })
        .send()
        .await
        .context("failed to complete git request")?
        .error_for_status()
        .context("relay rejected git completion")?;

    Ok(())
}

fn resolve_git_workspace_root(working_root: &Path, session_cwd: Option<&str>) -> Result<PathBuf> {
    let root = resolve_task_cwd(working_root, session_cwd);
    ensure_task_cwd(&root)
}

fn resolve_repo_root(workspace_root: &Path) -> Result<Option<PathBuf>> {
    let output = run_git(workspace_root, ["rev-parse", "--show-toplevel"])?;
    if output.status.success() {
        let repo_root = String::from_utf8(output.stdout)
            .context("git rev-parse returned non-utf8 repo root")?
            .trim()
            .to_string();
        let repo_root = PathBuf::from(repo_root)
            .canonicalize()
            .context("failed to canonicalize git repo root")?;
        return Ok(Some(repo_root));
    }

    if git_output_mentions_not_repo(&output) {
        return Ok(None);
    }

    bail!("git rev-parse failed: {}", render_git_failure(&output))
}

fn workspace_scope_path(repo_root: &Path, workspace_root: &Path) -> Result<Option<String>> {
    let scope = workspace_root
        .strip_prefix(repo_root)
        .with_context(|| {
            format!(
                "workspace root {} is outside git repo root {}",
                workspace_root.display(),
                repo_root.display()
            )
        })?
        .to_path_buf();

    if scope.as_os_str().is_empty() {
        Ok(None)
    } else {
        Ok(Some(scope.to_string_lossy().to_string()))
    }
}

fn run_git<I, S>(cwd: &Path, args: I) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = args
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect::<Vec<_>>();
    let output = StdCommand::new("git")
        .arg("-C")
        .arg(cwd)
        .args(&args)
        .output()
        .with_context(|| format!("failed to run git {} in {}", args.join(" "), cwd.display()))?;

    Ok(output)
}

fn git_args_with_scope(base_args: &[&str], scope_path: Option<&str>) -> Vec<String> {
    let mut args = base_args
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    if let Some(scope_path) = scope_path {
        args.push("--".to_string());
        args.push(scope_path.to_string());
    }
    args
}

fn ensure_git_success(output: &Output, command_name: &str) -> Result<()> {
    if output.status.success() {
        Ok(())
    } else {
        bail!("git {command_name} failed: {}", render_git_failure(output))
    }
}

fn git_output_mentions_not_repo(output: &Output) -> bool {
    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    stderr.contains("not a git repository")
}

fn render_git_failure(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!("exit status {}", output.status)
    }
}

fn parse_git_status(
    stdout: &[u8],
    repo_root: &Path,
) -> Result<(ParsedBranchState, Vec<GitChangedFile>, GitDiffStats)> {
    let mut branch_state = ParsedBranchState::default();
    let mut changed_files = Vec::new();
    let mut diff_stats = GitDiffStats::default();

    for record in stdout
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let record = String::from_utf8_lossy(record);
        if let Some(line) = record.strip_prefix("## ") {
            branch_state = parse_branch_line(line);
            continue;
        }

        if record.len() < 4 {
            continue;
        }

        let mut chars = record.chars();
        let index_status = chars.next().unwrap_or(' ');
        let worktree_status = chars.next().unwrap_or(' ');
        let path = record.get(3..).unwrap_or("").to_string();
        let status = derive_git_file_status(index_status, worktree_status);
        let staged = index_status != ' ' && index_status != '?';
        let unstaged = worktree_status != ' ' && worktree_status != '?';

        if staged {
            diff_stats.staged_files += 1;
        }
        if unstaged {
            diff_stats.unstaged_files += 1;
        }
        if matches!(status, GitFileStatus::Untracked) {
            diff_stats.untracked_files += 1;
        }
        if matches!(status, GitFileStatus::UpdatedButUnmerged) {
            diff_stats.conflicted_files += 1;
        }

        changed_files.push(GitChangedFile {
            path: repo_root.join(&path).display().to_string(),
            repo_path: path,
            status,
            staged,
            unstaged,
        });
    }

    changed_files.sort_by(|left, right| left.repo_path.cmp(&right.repo_path));
    diff_stats.changed_files = changed_files.len() as u64;

    Ok((branch_state, changed_files, diff_stats))
}

fn parse_branch_line(line: &str) -> ParsedBranchState {
    let mut state = ParsedBranchState::default();

    if let Some(branch_name) = line.strip_prefix("No commits yet on ") {
        state.branch_name = Some(branch_name.to_string());
        return state;
    }

    let (branch_part, ahead_behind_part) = match line.split_once(" [") {
        Some((left, right)) => (left, Some(right.trim_end_matches(']'))),
        None => (line, None),
    };

    if branch_part == "HEAD (no branch)" {
        state.branch_name = Some("HEAD".to_string());
    } else if let Some((branch_name, upstream_branch)) = branch_part.split_once("...") {
        state.branch_name = Some(branch_name.to_string());
        state.upstream_branch = Some(upstream_branch.to_string());
    } else if !branch_part.trim().is_empty() {
        state.branch_name = Some(branch_part.to_string());
    }

    if let Some(ahead_behind_part) = ahead_behind_part {
        for part in ahead_behind_part.split(", ") {
            if let Some(value) = part.strip_prefix("ahead ") {
                state.ahead_count = value.parse().unwrap_or(0);
            } else if let Some(value) = part.strip_prefix("behind ") {
                state.behind_count = value.parse().unwrap_or(0);
            }
        }
    }

    state
}

fn derive_git_file_status(index_status: char, worktree_status: char) -> GitFileStatus {
    if is_conflicted_status(index_status, worktree_status) {
        return GitFileStatus::UpdatedButUnmerged;
    }

    let status_code = if index_status != ' ' {
        index_status
    } else {
        worktree_status
    };

    match status_code {
        'M' => GitFileStatus::Modified,
        'A' => GitFileStatus::Added,
        'D' => GitFileStatus::Deleted,
        'R' => GitFileStatus::Renamed,
        'C' => GitFileStatus::Copied,
        'T' => GitFileStatus::TypeChanged,
        '?' => GitFileStatus::Untracked,
        _ => GitFileStatus::Unknown,
    }
}

fn is_conflicted_status(index_status: char, worktree_status: char) -> bool {
    matches!(
        (index_status, worktree_status),
        ('D', 'D') | ('A', 'U') | ('U', 'D') | ('U', 'A') | ('D', 'U') | ('A', 'A') | ('U', 'U')
    )
}

fn populate_numstat<I, S>(
    repo_root: &Path,
    args: I,
    additions: &mut u64,
    deletions: &mut u64,
) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let output = run_git(repo_root, args)?;
    ensure_git_success(&output, "diff")?;

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let mut parts = line.splitn(3, '\t');
        let added = parts.next().unwrap_or("");
        let deleted = parts.next().unwrap_or("");
        if let Ok(value) = added.parse::<u64>() {
            *additions += value;
        }
        if let Ok(value) = deleted.parse::<u64>() {
            *deletions += value;
        }
    }

    Ok(())
}

fn git_head_exists(repo_root: &Path) -> Result<bool> {
    let output = run_git(repo_root, ["rev-parse", "--verify", "HEAD"])?;
    Ok(output.status.success())
}

fn load_recent_commits(repo_root: &Path) -> Result<Vec<GitCommitSummary>> {
    let output = run_git(
        repo_root,
        [
            "log",
            "--format=%H%x09%h%x09%an%x09%ct%x09%s",
            "-n",
            &GIT_RECENT_COMMITS_LIMIT.to_string(),
        ],
    )?;
    ensure_git_success(&output, "log")?;

    let mut commits = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let mut parts = line.splitn(5, '\t');
        let id = parts.next().unwrap_or("").to_string();
        let short_id = parts.next().unwrap_or("").to_string();
        let author_name = parts.next().unwrap_or("").to_string();
        let committed_at_epoch_ms = parts
            .next()
            .and_then(|value| value.parse::<u64>().ok())
            .map(|value| value.saturating_mul(1_000))
            .unwrap_or(0);
        let summary = parts.next().unwrap_or("").to_string();

        if id.is_empty() || short_id.is_empty() {
            continue;
        }

        commits.push(GitCommitSummary {
            id,
            short_id,
            summary,
            author_name,
            committed_at_epoch_ms,
        });
    }

    Ok(commits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn temp_git_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("vibe-git-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn run_git_test(cwd: &Path, args: &[&str]) {
        let output = StdCommand::new("git")
            .arg("-C")
            .arg(cwd)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            render_git_failure(&output)
        );
    }

    #[test]
    fn parse_branch_line_reads_upstream_and_drift() {
        let parsed = parse_branch_line("main...origin/main [ahead 2, behind 1]");

        assert_eq!(parsed.branch_name.as_deref(), Some("main"));
        assert_eq!(parsed.upstream_branch.as_deref(), Some("origin/main"));
        assert_eq!(parsed.ahead_count, 2);
        assert_eq!(parsed.behind_count, 1);
    }

    #[test]
    fn handle_git_inspect_reports_non_repository() {
        let root = temp_git_dir();
        let response = handle_git_inspect("device-1", &root, None).unwrap();

        assert_eq!(response.state, GitInspectState::NotRepository);
        assert_eq!(response.workspace_root, root.display().to_string());
        assert!(response.changed_files.is_empty());
    }

    #[test]
    fn handle_git_inspect_collects_changed_files_and_commits() {
        if which::which("git").is_err() {
            return;
        }

        let root = temp_git_dir();
        run_git_test(&root, &["init"]);
        run_git_test(&root, &["config", "user.name", "Vibe Test"]);
        run_git_test(&root, &["config", "user.email", "vibe@example.com"]);

        fs::write(root.join("tracked.txt"), "one\n").unwrap();
        run_git_test(&root, &["add", "tracked.txt"]);
        run_git_test(&root, &["commit", "-m", "initial"]);

        fs::write(root.join("tracked.txt"), "one\nchanged\n").unwrap();
        fs::write(root.join("new.txt"), "new\n").unwrap();

        let response = handle_git_inspect("device-1", &root, None).unwrap();

        assert_eq!(response.state, GitInspectState::Ready);
        assert!(response.has_commits);
        assert!(
            response
                .recent_commits
                .iter()
                .any(|commit| commit.summary == "initial")
        );
        assert!(
            response
                .changed_files
                .iter()
                .any(|file| file.repo_path == "tracked.txt"
                    && file.status == GitFileStatus::Modified)
        );
        assert!(
            response
                .changed_files
                .iter()
                .any(|file| file.repo_path == "new.txt" && file.status == GitFileStatus::Untracked)
        );
        assert!(response.diff_stats.changed_files >= 2);
        assert!(response.diff_stats.untracked_files >= 1);
        assert!(response.diff_stats.unstaged_files >= 1);
    }
}
