use tempfile::TempDir;
use std::process::Command;

#[test]
fn test_sweep_finds_branches() {
    let dir = TempDir::new().unwrap();
    Command::new("git").args(["init"]).current_dir(dir.path()).output().unwrap();
    Command::new("git").args(["config", "user.email", "t@t.com"]).current_dir(dir.path()).output().unwrap();
    Command::new("git").args(["config", "user.name", "T"]).current_dir(dir.path()).output().unwrap();
    Command::new("git").args(["commit", "--allow-empty", "-m", "init"]).current_dir(dir.path()).output().unwrap();
    Command::new("git").args(["branch", "stale-branch"]).current_dir(dir.path()).output().unwrap();
    let branches = tinstar::git::list_branches(dir.path()).unwrap();
    assert!(branches.len() >= 2);
}

#[test]
fn test_sweep_prunes_old_leaks() {
    let dir = TempDir::new().unwrap();
    let leaks_dir = dir.path().join("leaks");
    std::fs::create_dir_all(&leaks_dir).unwrap();
    let old_leak = serde_json::json!({
        "id": "leak-old", "timestamp": "2025-01-01T00:00:00Z",
        "rule": "test", "command": "test", "hook": "PostToolUse",
        "project": "/tmp", "git_state": { "branch": "main", "dirty": false }
    });
    std::fs::write(leaks_dir.join("leak-old.json"), old_leak.to_string()).unwrap();
    let pruned = tinstar::commands::sweep::prune_old_leaks(&leaks_dir, 90);
    assert_eq!(pruned, 1);
}
