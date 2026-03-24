use std::process::Command;

fn tinstar(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tinstar"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn test_check_blocks_force_push() {
    let out = tinstar(&[
        "check",
        "--command",
        "git push --force origin main",
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("force-push"));
}

#[test]
fn test_check_allows_normal_push() {
    let out = tinstar(&["check", "--command", "git push origin main", "--json"]);
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn test_check_blocks_no_verify() {
    let out = tinstar(&[
        "check",
        "--command",
        "git commit --no-verify -m 'test'",
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("no-verify"));
}

#[test]
fn test_check_non_git_command_allowed() {
    let out = tinstar(&["check", "--command", "ls -la", "--json"]);
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn test_check_state_clean_repo_exits_valid() {
    let out = tinstar(&["check-state", "--json"]);
    // Must not exit 1 (internal error); 0 or 2 are valid
    assert_ne!(out.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("issues"));
}
