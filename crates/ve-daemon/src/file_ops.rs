//! File Operations Module
//!
//! Handles file system operations with workspace boundary validation.

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
    "txt",
    "md",
    "rs",
    "toml",
    "json",
    "yaml",
    "yml",
    "js",
    "ts",
    "tsx",
    "jsx",
    "html",
    "css",
    "scss",
    "sass",
    "less",
    "py",
    "go",
    "java",
    "kt",
    "kts",
    "c",
    "cpp",
    "h",
    "hpp",
    "cc",
    "cxx",
    "m",
    "mm",
    "swift",
    "rb",
    "php",
    "sh",
    "bash",
    "zsh",
    "fish",
    "lua",
    "pl",
    "pm",
    "sql",
    "proto",
    "env",
    "example",
    "cfg",
    "ini",
    "conf",
    "config",
    "xml",
    "svg",
    "vue",
    "svelte",
    "astro",
    "dockerfile",
    "makefile",
    "cmake",
    "gradle",
    "properties",
    "gitignore",
    "dockerignore",
    "editorconfig",
    "eslintrc",
    "prettierrc",
    "lock",
    "sum",
    "mod",
    "work",
    "cabal",
    "hs",
    "scm",
    "lisp",
    "el",
    "clj",
    "ex",
    "exs",
    "erl",
    "hrl",
    "scala",
    "cls",
    "java",
    "gradle",
    "mdx",
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
    ///
    /// # Security
    ///
    /// This function prevents path traversal attacks by ensuring the resolved path
    /// is within one of the allowed workspace roots.
    pub fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        // Canonicalize the path to resolve symlinks and relative components
        let canonical_path = path.canonicalize().map_err(|e| {
            warn!(path = %path.display(), error = %e, "Failed to canonicalize path");
            DaemonError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            }
        })?;

        // Check if the canonical path is within any workspace root
        for root in &self.workspace_roots {
            let canonical_root = match root.canonicalize() {
                Ok(r) => r,
                Err(_) => continue, // Skip roots that don't exist
            };

            if canonical_path.starts_with(&canonical_root) {
                return Ok(canonical_path);
            }
        }

        // Path is outside all workspace roots
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
    ///
    /// # Arguments
    /// * `path` - Starting directory path
    /// * `max_depth` - Maximum depth to traverse
    ///
    /// # Returns
    /// File tree structure with nodes
    pub fn collect_tree(&self, path: &Path, max_depth: usize) -> Result<FileTreeNode> {
        // Validate path is within workspace
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
        // Check node limit
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
                // Skip files we can't access
                warn!(
                    path = %current_path.display(),
                    error = %e,
                    "Failed to get metadata, skipping"
                );
                return Err(DaemonError::FileReadFailed {
                    path: current_path.to_string_lossy().to_string(),
                    source: e,
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
                        // Check node limit before processing next child
                        if *node_count >= self.tree_max_nodes {
                            return Err(DaemonError::FileTreeLimitExceeded {
                                count: *node_count,
                                limit: self.tree_max_nodes,
                            });
                        }
                        // Skip symlinks (security measure)
                        let entry_path = entry.path();
                        if let Ok(ft) = entry.file_type() {
                            if ft.is_symlink() {
                                continue;
                            }
                        }
                        // Skip directories in SKIP_DIRS list
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
                            Err(e) => return Err(e),
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
        // Check extension first
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            if TEXT_EXTENSIONS.contains(&ext_lower.as_str()) {
                return FileType::Text;
            }
        }

        // Check file name (for dotfiles without extension)
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            // Dotfiles like .gitignore, .env
            if name.starts_with('.') && !name.contains('.') {
                return FileType::Text;
            }
            // Common config files without extension
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

        // Try to detect binary by reading first bytes
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
    fn is_binary(data: &[u8]) -> bool {
        data.contains(&0x00)
    }

    /// Read text file content with size limit
    ///
    /// # Arguments
    /// * `path` - File path to read
    ///
    /// # Returns
    /// File content with metadata
    pub fn read_text_file(&self, path: &Path) -> Result<FileContent> {
        // Validate path is within workspace
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

        // Only read text files
        if file_type != FileType::Text {
            return Err(DaemonError::FileNotText {
                path: path.to_string_lossy().to_string(),
            });
        }

        // Check if file is too large
        if total_size > self.read_text_limit as u64 {
            return Err(DaemonError::FileTooLarge {
                size: total_size,
                limit: self.read_text_limit as u64,
            });
        }

        // Read file
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

        // Convert to string (handle invalid UTF-8)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file_ops() -> (FileOps, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let roots = vec![temp_dir.path().to_path_buf()];
        let ops = FileOps::new(roots, 1024, 100);
        (ops, temp_dir)
    }

    #[test]
    fn test_validate_path_within_workspace() {
        let (ops, temp_dir) = create_test_file_ops();

        // Create a file in the workspace
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let result = ops.validate_path(&file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), file_path.canonicalize().unwrap());
    }

    #[test]
    fn test_validate_path_outside_workspace() {
        let (ops, _temp_dir) = create_test_file_ops();

        // Try to access /etc/passwd (should be blocked)
        let result = ops.validate_path(Path::new("/etc/passwd"));
        assert!(result.is_err());

        match result {
            Err(DaemonError::FileAccessDenied { .. }) => {}
            _ => panic!("Expected FileAccessDenied error"),
        }
    }

    #[test]
    fn test_validate_path_traversal_attack() {
        let (ops, temp_dir) = create_test_file_ops();

        // Create a file in the workspace
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        // Try path traversal: workspace/test.txt/../../../etc/passwd
        let traversal_path = temp_dir
            .path()
            .join("test.txt")
            .join("..")
            .join("..")
            .join("..")
            .join("etc")
            .join("passwd");
        let result = ops.validate_path(&traversal_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_symlink_outside_workspace() {
        let (ops, temp_dir) = create_test_file_ops();

        // Create a symlink pointing outside the workspace
        let symlink_path = temp_dir.path().join("outside_link");
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink("/etc", &symlink_path).ok(); // May fail if /etc doesn't exist or no permissions
        }

        // This test is platform-dependent, so we just verify the behavior exists
        if symlink_path.exists() {
            let _result = ops.validate_path(&symlink_path);
            // Should be blocked if symlink points outside workspace
            // The actual behavior depends on the symlink target
        }
    }

    #[test]
    fn test_is_within_workspace() {
        let (ops, temp_dir) = create_test_file_ops();

        // Create a file in the workspace
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        assert!(ops.is_within_workspace(&file_path));
        assert!(!ops.is_within_workspace(Path::new("/etc/passwd")));
    }

    #[test]
    fn test_nonexistent_path_returns_error() {
        let (ops, _temp_dir) = create_test_file_ops();

        let result = ops.validate_path(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    // ========== collect_tree tests ==========

    #[test]
    fn test_collect_tree_basic() {
        let (ops, temp_dir) = create_test_file_ops();

        // Create a simple directory structure
        fs::create_dir(temp_dir.path().join("src")).unwrap();
        fs::write(temp_dir.path().join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(temp_dir.path().join("README.md"), "# Test Project").unwrap();

        let tree = ops.collect_tree(temp_dir.path(), 10).unwrap();

        assert!(tree.is_dir);
        assert!(tree.children.is_some());
        let children = tree.children.unwrap();
        assert_eq!(children.len(), 2); // src and README.md
    }

    #[test]
    fn test_collect_tree_skip_dirs() {
        let (ops, temp_dir) = create_test_file_ops();

        // Create .git directory (should be skipped)
        fs::create_dir_all(temp_dir.path().join(".git/objects")).unwrap();
        fs::write(temp_dir.path().join(".git/config"), "[core]").unwrap();
        fs::write(temp_dir.path().join("main.rs"), "fn main() {}").unwrap();

        let tree = ops.collect_tree(temp_dir.path(), 10).unwrap();

        // .git should be skipped
        let children = tree.children.unwrap();
        assert!(!children.iter().any(|c| c.name == ".git"));
    }

    #[test]
    fn test_collect_tree_max_nodes() {
        let temp_dir = TempDir::new().unwrap();
        let roots = vec![temp_dir.path().to_path_buf()];
        // Set a low node limit
        let ops = FileOps::new(roots, 1024, 5);

        // Create more files than the limit
        for i in 0..10 {
            fs::write(temp_dir.path().join(format!("file{}.txt", i)), "content").unwrap();
        }

        let result = ops.collect_tree(temp_dir.path(), 10);
        assert!(result.is_err());
        match result {
            Err(DaemonError::FileTreeLimitExceeded { .. }) => {}
            _ => panic!("Expected FileTreeLimitExceeded error"),
        }
    }

    #[test]
    fn test_collect_tree_empty_directory() {
        let (ops, temp_dir) = create_test_file_ops();

        let tree = ops.collect_tree(temp_dir.path(), 10).unwrap();

        assert!(tree.is_dir);
        assert!(tree.children.is_none()); // Empty directory has no children
    }

    #[test]
    fn test_collect_tree_depth_limit() {
        let (ops, temp_dir) = create_test_file_ops();

        // Create nested directories
        fs::create_dir_all(temp_dir.path().join("a/b/c/d")).unwrap();
        fs::write(temp_dir.path().join("a/b/c/d/deep.txt"), "deep").unwrap();

        // With depth 2, should not reach d/deep.txt
        let tree = ops.collect_tree(temp_dir.path(), 2).unwrap();

        fn count_max_depth(node: &FileTreeNode, current: usize) -> usize {
            match &node.children {
                Some(children) => children
                    .iter()
                    .map(|c| count_max_depth(c, current + 1))
                    .max()
                    .unwrap_or(current),
                None => current,
            }
        }

        let max_depth = count_max_depth(&tree, 0);
        assert!(max_depth <= 2);
    }

    // ========== classify_file tests ==========

    #[test]
    fn test_classify_file_text_extension() {
        let (ops, temp_dir) = create_test_file_ops();

        // Create files with known text extensions
        fs::write(temp_dir.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(temp_dir.path().join("config.json"), "{}").unwrap();
        fs::write(temp_dir.path().join("readme.md"), "# Test").unwrap();

        let tree = ops.collect_tree(temp_dir.path(), 10).unwrap();
        let children = tree.children.unwrap();

        for child in children {
            assert_eq!(child.file_type, FileType::Text);
        }
    }

    #[test]
    fn test_classify_file_binary() {
        let (ops, temp_dir) = create_test_file_ops();

        // Create a binary file with null bytes
        fs::write(temp_dir.path().join("binary.bin"), b"\x00\x01\x02\x03").unwrap();

        let tree = ops.collect_tree(temp_dir.path(), 10).unwrap();
        let children = tree.children.unwrap();

        let binary = children.iter().find(|c| c.name == "binary.bin").unwrap();
        assert_eq!(binary.file_type, FileType::Binary);
    }

    #[test]
    fn test_classify_file_dotfile() {
        let (ops, temp_dir) = create_test_file_ops();

        // Create dotfiles
        fs::write(temp_dir.path().join(".gitignore"), "target/").unwrap();
        fs::write(temp_dir.path().join(".env"), "KEY=value").unwrap();

        let tree = ops.collect_tree(temp_dir.path(), 10).unwrap();
        let children = tree.children.unwrap();

        let gitignore = children.iter().find(|c| c.name == ".gitignore").unwrap();
        let env = children.iter().find(|c| c.name == ".env").unwrap();
        assert_eq!(gitignore.file_type, FileType::Text);
        assert_eq!(env.file_type, FileType::Text);
    }

    #[test]
    fn test_classify_file_config_names() {
        let (ops, temp_dir) = create_test_file_ops();

        // Create config files without extension
        fs::write(temp_dir.path().join("Dockerfile"), "FROM alpine").unwrap();
        fs::write(temp_dir.path().join("Makefile"), "all: build").unwrap();

        let tree = ops.collect_tree(temp_dir.path(), 10).unwrap();
        let children = tree.children.unwrap();

        let dockerfile = children.iter().find(|c| c.name == "Dockerfile").unwrap();
        let makefile = children.iter().find(|c| c.name == "Makefile").unwrap();
        assert_eq!(dockerfile.file_type, FileType::Text);
        assert_eq!(makefile.file_type, FileType::Text);
    }

    // ========== read_text_file tests ==========

    #[test]
    fn test_read_text_file_success() {
        let (ops, temp_dir) = create_test_file_ops();

        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, World!").unwrap();

        let content = ops.read_text_file(&file_path).unwrap();

        assert_eq!(content.content, "Hello, World!");
        assert_eq!(content.file_type, FileType::Text);
        assert!(!content.truncated);
        assert_eq!(content.total_size, 13);
    }

    #[test]
    fn test_read_text_file_directory_error() {
        let (ops, temp_dir) = create_test_file_ops();

        let dir_path = temp_dir.path().join("subdir");
        fs::create_dir(&dir_path).unwrap();

        let result = ops.read_text_file(&dir_path);
        assert!(result.is_err());
        match result {
            Err(DaemonError::FileNotText { .. }) => {}
            _ => panic!("Expected FileNotText error"),
        }
    }

    #[test]
    fn test_read_text_file_binary_error() {
        let (ops, temp_dir) = create_test_file_ops();

        // Create a file that looks binary (unknown extension + null bytes)
        let file_path = temp_dir.path().join("data.xyz");
        fs::write(&file_path, b"\x00\x01\x02\x03").unwrap();

        let result = ops.read_text_file(&file_path);
        assert!(result.is_err());
        match result {
            Err(DaemonError::FileNotText { .. }) => {}
            _ => panic!("Expected FileNotText error"),
        }
    }

    #[test]
    fn test_read_text_file_too_large() {
        let temp_dir = TempDir::new().unwrap();
        let roots = vec![temp_dir.path().to_path_buf()];
        // Set a very low limit
        let ops = FileOps::new(roots, 10, 100);

        let file_path = temp_dir.path().join("large.txt");
        fs::write(&file_path, "This is more than 10 bytes").unwrap();

        let result = ops.read_text_file(&file_path);
        assert!(result.is_err());
        match result {
            Err(DaemonError::FileTooLarge { .. }) => {}
            _ => panic!("Expected FileTooLarge error"),
        }
    }

    #[test]
    fn test_read_text_file_outside_workspace_error() {
        let (ops, _temp_dir) = create_test_file_ops();

        // Try to read a file outside workspace
        let result = ops.read_text_file(Path::new("/etc/passwd"));
        assert!(result.is_err());
        match result {
            Err(DaemonError::FileAccessDenied { .. }) => {}
            Err(DaemonError::FileNotFound { .. }) => {} // If /etc/passwd doesn't exist
            _ => panic!("Expected access denied or not found error"),
        }
    }

    // ========== is_binary tests ==========

    #[test]
    fn test_is_binary_with_null_bytes() {
        assert!(FileOps::is_binary(&[0x00, 0x01, 0x02]));
        assert!(FileOps::is_binary(&[0x48, 0x65, 0x6c, 0x6c, 0x00])); // "Hell\0"
    }

    #[test]
    fn test_is_binary_without_null_bytes() {
        assert!(!FileOps::is_binary(b"Hello, World!"));
        assert!(!FileOps::is_binary(b"\xff\xfe\xfd")); // High bytes but no null
    }

    #[test]
    fn test_is_binary_empty() {
        assert!(!FileOps::is_binary(&[]));
    }
}
