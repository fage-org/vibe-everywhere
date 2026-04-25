//! File operation handlers with workspace boundary validation.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use tracing::warn;
use ve_shared::models::{FileContent, FileTreeNode, FileType};

use crate::error::DaemonError;
use crate::Result;

/// Directories to skip when building file tree
const SKIP_DIRS: &[&str] = &[
    ".git",
    ".svn",
    ".hg",
    "node_modules",
    "target",
    "build",
    "dist",
    ".cache",
    "__pycache__",
    ".venv",
    "venv",
    ".idea",
    ".vscode",
    ".next",
    ".nuxt",
    "vendor",
    "Pods",
];

/// File extensions that are typically text files
const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "rs", "toml", "json", "yaml", "yml", "js", "ts", "tsx", "jsx", "html", "css",
    "scss", "sass", "less", "py", "go", "java", "kt", "kts", "c", "cpp", "h", "hpp", "cc", "cxx",
    "m", "mm", "swift", "rb", "php", "sh", "bash", "zsh", "fish", "lua", "pl", "pm", "sql",
    "proto", "env", "example", "cfg", "ini", "conf", "config", "xml", "svg", "vue", "svelte",
    "astro", "dockerfile", "makefile", "cmake", "gradle", "properties", "gitignore",
    "dockerignore", "editorconfig", "eslintrc", "prettierrc", "lock", "sum", "mod", "work",
    "cabal", "hs", "scm", "lisp", "el", "clj", "ex", "exs", "erl", "hrl", "scala", "cls", "java",
    "gradle", "mdx",
];

/// File operations handler with workspace boundary validation
#[derive(Clone)]
pub struct FileOps {
    /// Allowed workspace roots
    workspace_roots: Vec<PathBuf>,
    /// Maximum file size for text reading (bytes)
    read_text_limit: usize,
    /// Maximum nodes for file tree
    tree_max_nodes: usize,
}

impl FileOps {
    /// Create a new file operations handler
    pub fn new(
        workspace_roots: Vec<PathBuf>,
        read_text_limit: usize,
        tree_max_nodes: usize,
    ) -> Self {
        Self {
            workspace_roots,
            read_text_limit,
            tree_max_nodes,
        }
    }

    /// Validate that a path is within allowed workspace boundaries
    pub fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        let canonical_path = path.canonicalize().map_err(|e| {
            warn!(path = %path.display(), error = %e, "Failed to canonicalize path");
            DaemonError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            }
        })?;

        for root in &self.workspace_roots {
            let canonical_root = match root.canonicalize() {
                Ok(r) => r,
                Err(_) => continue,
            };

            if canonical_path.starts_with(&canonical_root) {
                return Ok(canonical_path);
            }
        }

        let first_root = self
            .workspace_roots
            .first()
            .map(|r| r.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        warn!(
            path = %canonical_path.display(),
            roots = ?self.workspace_roots,
            "Path traversal attempt blocked"
        );
        Err(DaemonError::FileAccessDenied {
            path: canonical_path.to_string_lossy().to_string(),
            workspace: first_root,
        })
    }

    /// Check if a path is within workspace boundaries (returns bool)
    pub fn is_within_workspace(&self, path: &Path) -> bool {
        self.validate_path(path).is_ok()
    }

    /// Get the read text limit
    pub fn read_text_limit(&self) -> usize {
        self.read_text_limit
    }

    /// Get the tree max nodes
    pub fn tree_max_nodes(&self) -> usize {
        self.tree_max_nodes
    }

    /// Collect file tree from a directory
    pub fn collect_tree(&self, path: &Path, max_depth: usize) -> Result<FileTreeNode> {
        let validated_path = self.validate_path(path)?;

        let mut node_count = 0usize;
        self.collect_tree_internal(&validated_path, &validated_path, max_depth, &mut node_count)
    }

    fn collect_tree_internal(
        &self,
        current_path: &Path,
        root_path: &Path,
        remaining_depth: usize,
        node_count: &mut usize,
    ) -> Result<FileTreeNode> {
        if *node_count >= self.tree_max_nodes {
            return Err(DaemonError::FileTreeLimitExceeded {
                count: *node_count,
                limit: self.tree_max_nodes,
            });
        }
        *node_count += 1;

        let name = current_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let relative_path = current_path
            .strip_prefix(root_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let metadata = match fs::metadata(current_path) {
            Ok(m) => m,
            Err(e) => {
                warn!(
                    path = %current_path.display(),
                    error = %e,
                    "Failed to get metadata, skipping"
                );
                return Ok(FileTreeNode {
                    name,
                    path: relative_path,
                    is_dir: false,
                    file_type: FileType::Unknown,
                    size: None,
                    children: None,
                });
            }
        };

        let is_dir = metadata.is_dir();
        let file_type = if is_dir {
            FileType::Unknown
        } else {
            self.classify_file(current_path)
        };

        let children = if is_dir && remaining_depth > 0 {
            let mut child_nodes = Vec::new();
            match fs::read_dir(current_path) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        if *node_count >= self.tree_max_nodes {
                            return Err(DaemonError::FileTreeLimitExceeded {
                                count: *node_count,
                                limit: self.tree_max_nodes,
                            });
                        }
                        let entry_path = entry.path();
                        if let Ok(ft) = entry.file_type() {
                            if ft.is_symlink() {
                                continue;
                            }
                        }
                        let entry_name = entry.file_name();
                        let entry_name_str = entry_name.to_string_lossy();
                        if SKIP_DIRS.contains(&entry_name_str.as_ref()) {
                            continue;
                        }
                        match self.collect_tree_internal(
                            &entry_path,
                            root_path,
                            remaining_depth - 1,
                            node_count,
                        ) {
                            Ok(child) => child_nodes.push(child),
                            Err(e) => {
                                warn!(
                                    path = %entry_path.display(),
                                    error = %e,
                                    "Failed to collect child, skipping"
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        path = %current_path.display(),
                        error = %e,
                        "Failed to read directory"
                    );
                }
            }
            if child_nodes.is_empty() {
                None
            } else {
                Some(child_nodes)
            }
        } else {
            None
        };

        Ok(FileTreeNode {
            name,
            path: relative_path,
            is_dir,
            file_type,
            size: if is_dir { None } else { Some(metadata.len()) },
            children,
        })
    }

    /// Classify file as text, binary, or unknown
    fn classify_file(&self, path: &Path) -> FileType {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            if TEXT_EXTENSIONS.contains(&ext_lower.as_str()) {
                return FileType::Text;
            }
        }

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') && !name.contains('.') {
                return FileType::Text;
            }
            let config_names = [
                "Dockerfile",
                "Makefile",
                "CMakeLists",
                "Vagrantfile",
                "Gemfile",
                "Rakefile",
                "Procfile",
                "Jenkinsfile",
                "Vagrantfile",
            ];
            if config_names.contains(&name) {
                return FileType::Text;
            }
        }

        if let Ok(mut file) = fs::File::open(path) {
            let mut buffer = [0u8; 512];
            if let Ok(n) = file.read(&mut buffer[..]) {
                if n > 0 {
                    if Self::is_binary(&buffer[..n]) {
                        return FileType::Binary;
                    }
                    return FileType::Text;
                }
            }
        }

        FileType::Unknown
    }

    /// Detect if content is binary by checking for null bytes
    pub(crate) fn is_binary(data: &[u8]) -> bool {
        data.contains(&0x00)
    }

    /// Read text file content with size limit
    pub fn read_text_file(&self, path: &Path) -> Result<FileContent> {
        let validated_path = self.validate_path(path)?;

        let metadata = fs::metadata(&validated_path).map_err(|e| DaemonError::FileReadFailed {
            path: path.to_string_lossy().to_string(),
            source: e,
        })?;

        if metadata.is_dir() {
            return Err(DaemonError::FileNotText {
                path: path.to_string_lossy().to_string(),
            });
        }

        let total_size = metadata.len();
        let file_type = self.classify_file(&validated_path);

        if file_type != FileType::Text {
            return Err(DaemonError::FileNotText {
                path: path.to_string_lossy().to_string(),
            });
        }

        let mut file =
            fs::File::open(&validated_path).map_err(|e| DaemonError::FileReadFailed {
                path: path.to_string_lossy().to_string(),
                source: e,
            })?;

        let read_size = std::cmp::min(total_size, self.read_text_limit as u64) as usize;
        let truncated = total_size > self.read_text_limit as u64;

        let mut buffer = vec![0u8; read_size];
        file.read_exact(&mut buffer)
            .map_err(|e| DaemonError::FileReadFailed {
                path: path.to_string_lossy().to_string(),
                source: e,
            })?;

        let content = String::from_utf8_lossy(&buffer).to_string();

        Ok(FileContent {
            path: path.to_string_lossy().to_string(),
            content,
            file_type,
            truncated,
            total_size,
        })
    }
}
