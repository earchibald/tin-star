use tempfile::TempDir;

#[test]
fn test_issue_create_writes_file() {
    let dir = TempDir::new().unwrap();
    let leaks_dir = dir.path().join("leaks");
    tinstar::commands::issue::create_leak(
        &leaks_dir, "force-push", "git push --force origin main",
        "PostToolUse", "/tmp/project", "main", false,
    );
    let files: Vec<_> = std::fs::read_dir(&leaks_dir).unwrap().collect();
    assert_eq!(files.len(), 1);
    let content = std::fs::read_to_string(files[0].as_ref().unwrap().path()).unwrap();
    assert!(content.contains("force-push"));
}

#[test]
fn test_issue_list_reads_files() {
    let dir = TempDir::new().unwrap();
    let leaks_dir = dir.path().join("leaks");
    tinstar::commands::issue::create_leak(&leaks_dir, "force-push", "git push -f", "PostToolUse", "/tmp/a", "main", false);
    tinstar::commands::issue::create_leak(&leaks_dir, "no-verify", "git commit --no-verify", "PostToolUse", "/tmp/b", "feat/x", false);
    let leaks = tinstar::commands::issue::list_leaks(&leaks_dir);
    assert_eq!(leaks.len(), 2);
}

#[test]
fn test_issue_uuid_uniqueness() {
    let dir = TempDir::new().unwrap();
    let leaks_dir = dir.path().join("leaks");
    tinstar::commands::issue::create_leak(&leaks_dir, "test", "cmd1", "PostToolUse", "/tmp", "main", false);
    tinstar::commands::issue::create_leak(&leaks_dir, "test", "cmd2", "PostToolUse", "/tmp", "main", false);
    let files: Vec<_> = std::fs::read_dir(&leaks_dir).unwrap().collect();
    assert_eq!(files.len(), 2);
    let names: Vec<String> = files.iter().map(|f| f.as_ref().unwrap().file_name().to_string_lossy().to_string()).collect();
    assert_ne!(names[0], names[1]);
}
