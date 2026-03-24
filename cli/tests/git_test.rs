use std::process::Command;
use tempfile::TempDir;

fn init_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    Command::new("git").args(["init"]).current_dir(dir.path()).output().unwrap();
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(dir.path()).output().unwrap();
    Command::new("git").args(["config", "user.name", "Test"]).current_dir(dir.path()).output().unwrap();
    Command::new("git").args(["commit", "--allow-empty", "-m", "init"])
        .current_dir(dir.path()).output().unwrap();
    dir
}

#[test]
fn test_current_branch() {
    let dir = init_repo();
    let branch = tinstar::git::current_branch(dir.path()).unwrap();
    assert!(branch == "main" || branch == "master");
}

#[test]
fn test_is_dirty_clean_repo() {
    let dir = init_repo();
    assert!(!tinstar::git::is_dirty(dir.path()));
}

#[test]
fn test_is_dirty_with_changes() {
    let dir = init_repo();
    std::fs::write(dir.path().join("file.txt"), "content").unwrap();
    assert!(tinstar::git::is_dirty(dir.path()));
}

#[test]
fn test_staged_diff_empty() {
    let dir = init_repo();
    let diff = tinstar::git::staged_diff(dir.path()).unwrap();
    assert!(diff.is_empty());
}

#[test]
fn test_staged_diff_with_content() {
    let dir = init_repo();
    std::fs::write(dir.path().join("secret.txt"), "api_key = 'test123'").unwrap();
    Command::new("git").args(["add", "secret.txt"]).current_dir(dir.path()).output().unwrap();
    let diff = tinstar::git::staged_diff(dir.path()).unwrap();
    assert!(diff.contains("api_key"));
}

#[test]
fn test_is_detached_head() {
    let dir = init_repo();
    assert!(!tinstar::git::is_detached_head(dir.path()));
}

#[test]
fn test_extract_commit_message_double_quotes() {
    let msg = tinstar::git::extract_commit_message(r#"git commit -m "fix: resolve bug""#);
    assert_eq!(msg, Some("fix: resolve bug".to_string()));
}

#[test]
fn test_extract_commit_message_single_quotes() {
    let msg = tinstar::git::extract_commit_message("git commit -m 'feat: add feature'");
    assert_eq!(msg, Some("feat: add feature".to_string()));
}

#[test]
fn test_extract_commit_message_none() {
    let msg = tinstar::git::extract_commit_message("git commit --amend");
    assert_eq!(msg, None);
}

#[test]
fn test_working_diff_empty() {
    let dir = init_repo();
    let diff = tinstar::git::working_diff(dir.path()).unwrap();
    assert!(diff.is_empty());
}

#[test]
fn test_working_diff_with_changes() {
    let dir = init_repo();
    // Create and commit a file first
    std::fs::write(dir.path().join("file.txt"), "original").unwrap();
    Command::new("git").args(["add", "file.txt"]).current_dir(dir.path()).output().unwrap();
    Command::new("git").args(["commit", "-m", "add file"]).current_dir(dir.path()).output().unwrap();
    // Now modify it
    std::fs::write(dir.path().join("file.txt"), "modified").unwrap();
    let diff = tinstar::git::working_diff(dir.path()).unwrap();
    assert!(diff.contains("modified"));
}

#[test]
fn test_commits_behind_upstream_no_upstream() {
    let dir = init_repo();
    let result = tinstar::git::commits_behind_upstream(dir.path()).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_list_branches() {
    let dir = init_repo();
    let branches = tinstar::git::list_branches(dir.path()).unwrap();
    assert!(!branches.is_empty());
    // The default branch should be listed
    assert!(branches.iter().any(|b| b.name == "main" || b.name == "master"));
}
