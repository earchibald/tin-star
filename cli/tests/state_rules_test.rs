use tinstar::rules::commit_message::CommitMessage;
use tinstar::rules::commit_to_main::CommitToMain;
use tinstar::rules::Rule;

#[test]
fn test_commit_to_main_detects_commit_command() {
    let rule = CommitToMain;
    let result = rule.check_with_branch("git commit -m 'test'", "main");
    assert!(result.is_some());
}

#[test]
fn test_commit_to_main_allows_feature_branch() {
    let rule = CommitToMain;
    let result = rule.check_with_branch("git commit -m 'test'", "feat/new-thing");
    assert!(result.is_none());
}

#[test]
fn test_commit_to_main_detects_master() {
    let rule = CommitToMain;
    let result = rule.check_with_branch("git commit -m 'test'", "master");
    assert!(result.is_some());
}

#[test]
fn test_commit_message_blocks_empty() {
    let rule = CommitMessage;
    let result = rule.check_command("git commit -m ''");
    assert!(result.is_some());
}

#[test]
fn test_commit_message_blocks_whitespace_only() {
    let rule = CommitMessage;
    let result = rule.check_command("git commit -m '   '");
    assert!(result.is_some());
}

#[test]
fn test_commit_message_allows_meaningful() {
    let rule = CommitMessage;
    let result = rule.check_command("git commit -m 'feat: add user login'");
    assert!(result.is_none());
}

#[test]
fn test_commit_no_message_flag_allowed() {
    let rule = CommitMessage;
    let result = rule.check_command("git commit");
    assert!(result.is_none());
}

// ---------------------------------------------------------------------------
// evaluate_all_rules integration test
// ---------------------------------------------------------------------------

use std::process::Command;
use tempfile::TempDir;
use tinstar::config::Config;
use tinstar::rules::{evaluate_all_rules, RuleResult};

fn init_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "t@t.com"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "T"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    dir
}

#[test]
fn test_evaluate_all_rules_composes_command_and_state_rules() {
    let dir = init_repo();
    let config = Config::default();
    let results = evaluate_all_rules("git commit --no-verify -m 'test'", &config, dir.path());
    let rule_names: Vec<&str> = results
        .iter()
        .filter_map(|r| match r {
            RuleResult::Block { rule, .. } | RuleResult::Warn { rule, .. } => Some(rule.as_str()),
            _ => None,
        })
        .collect();
    assert!(rule_names.contains(&"no-verify"));
    assert!(rule_names.contains(&"commit-to-main"));
}
