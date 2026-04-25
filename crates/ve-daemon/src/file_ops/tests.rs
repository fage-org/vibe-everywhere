//! Tests for the file_ops module.

use super::*;
use std::fs;
use std::path::Path;
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
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "test content").unwrap();
    let result = ops.validate_path(&file_path);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), file_path.canonicalize().unwrap());
}

#[test]
fn test_validate_path_outside_workspace() {
    let (ops, _temp_dir) = create_test_file_ops();
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
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "test content").unwrap();
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
    let symlink_path = temp_dir.path().join("outside_link");
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink("/etc", &symlink_path).ok();
    }
    if symlink_path.exists() {
        let _result = ops.validate_path(&symlink_path);
    }
}

#[test]
fn test_is_within_workspace() {
    let (ops, temp_dir) = create_test_file_ops();
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

#[test]
fn test_collect_tree_basic() {
    let (ops, temp_dir) = create_test_file_ops();
    fs::create_dir(temp_dir.path().join("src")).unwrap();
    fs::write(temp_dir.path().join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(temp_dir.path().join("README.md"), "# Test Project").unwrap();
    let tree = ops.collect_tree(temp_dir.path(), 10).unwrap();
    assert!(tree.is_dir);
    assert!(tree.children.is_some());
    let children = tree.children.unwrap();
    assert_eq!(children.len(), 2);
}

#[test]
fn test_collect_tree_skip_dirs() {
    let (ops, temp_dir) = create_test_file_ops();
    fs::create_dir_all(temp_dir.path().join(".git/objects")).unwrap();
    fs::write(temp_dir.path().join(".git/config"), "[core]").unwrap();
    fs::write(temp_dir.path().join("main.rs"), "fn main() {}").unwrap();
    let tree = ops.collect_tree(temp_dir.path(), 10).unwrap();
    let children = tree.children.unwrap();
    assert!(!children.iter().any(|c| c.name == ".git"));
}

#[test]
fn test_collect_tree_max_nodes() {
    let temp_dir = TempDir::new().unwrap();
    let roots = vec![temp_dir.path().to_path_buf()];
    let ops = FileOps::new(roots, 1024, 5);
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
    assert!(tree.children.is_none());
}

#[test]
fn test_collect_tree_depth_limit() {
    let (ops, temp_dir) = create_test_file_ops();
    fs::create_dir_all(temp_dir.path().join("a/b/c/d")).unwrap();
    fs::write(temp_dir.path().join("a/b/c/d/deep.txt"), "deep").unwrap();
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

#[test]
fn test_classify_file_text_extension() {
    let (ops, temp_dir) = create_test_file_ops();
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
    fs::write(temp_dir.path().join("binary.bin"), b"\x00\x01\x02\x03").unwrap();
    let tree = ops.collect_tree(temp_dir.path(), 10).unwrap();
    let children = tree.children.unwrap();
    let binary = children.iter().find(|c| c.name == "binary.bin").unwrap();
    assert_eq!(binary.file_type, FileType::Binary);
}

#[test]
fn test_classify_file_dotfile() {
    let (ops, temp_dir) = create_test_file_ops();
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
    fs::write(temp_dir.path().join("Dockerfile"), "FROM alpine").unwrap();
    fs::write(temp_dir.path().join("Makefile"), "all: build").unwrap();
    let tree = ops.collect_tree(temp_dir.path(), 10).unwrap();
    let children = tree.children.unwrap();
    let dockerfile = children.iter().find(|c| c.name == "Dockerfile").unwrap();
    let makefile = children.iter().find(|c| c.name == "Makefile").unwrap();
    assert_eq!(dockerfile.file_type, FileType::Text);
    assert_eq!(makefile.file_type, FileType::Text);
}

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
    let ops = FileOps::new(roots, 10, 100);
    let file_path = temp_dir.path().join("large.txt");
    fs::write(&file_path, "This is more than 10 bytes").unwrap();
    let result = ops.read_text_file(&file_path).unwrap();
    assert_eq!(result.content.len(), 10);
    assert_eq!(result.content, "This is mo");
    assert!(result.truncated);
    assert_eq!(result.total_size, 26);
}

#[test]
fn test_read_text_file_outside_workspace_error() {
    let (ops, _temp_dir) = create_test_file_ops();
    let result = ops.read_text_file(Path::new("/etc/passwd"));
    assert!(result.is_err());
    match result {
        Err(DaemonError::FileAccessDenied { .. }) => {}
        Err(DaemonError::FileNotFound { .. }) => {}
        _ => panic!("Expected access denied or not found error"),
    }
}

#[test]
fn test_is_binary_with_null_bytes() {
    assert!(FileOps::is_binary(&[0x00, 0x01, 0x02]));
    assert!(FileOps::is_binary(&[0x48, 0x65, 0x6c, 0x6c, 0x00]));
}

#[test]
fn test_is_binary_without_null_bytes() {
    assert!(!FileOps::is_binary(b"Hello, World!"));
    assert!(!FileOps::is_binary(b"\xff\xfe\xfd"));
}

#[test]
fn test_is_binary_empty() {
    assert!(!FileOps::is_binary(&[]));
}
