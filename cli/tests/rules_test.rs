use tinstar::config::Config;
use tinstar::rules::{evaluate_command_rules, RuleResult};

#[test]
fn test_force_push_blocked() {
    let config = Config::default();
    let results = evaluate_command_rules("git push --force origin main", &config);
    assert!(results
        .iter()
        .any(|r| matches!(r, RuleResult::Block { rule, .. } if rule == "force-push")));
}

#[test]
fn test_force_push_short_flag_blocked() {
    let config = Config::default();
    let results = evaluate_command_rules("git push -f origin main", &config);
    assert!(results
        .iter()
        .any(|r| matches!(r, RuleResult::Block { rule, .. } if rule == "force-push")));
}

#[test]
fn test_force_with_lease_allowed() {
    let config = Config::default();
    let results = evaluate_command_rules("git push --force-with-lease origin main", &config);
    assert!(!results
        .iter()
        .any(|r| matches!(r, RuleResult::Block { rule, .. } if rule == "force-push")));
}

#[test]
fn test_no_verify_blocked() {
    let config = Config::default();
    let results = evaluate_command_rules("git commit --no-verify -m 'test'", &config);
    assert!(results
        .iter()
        .any(|r| matches!(r, RuleResult::Block { rule, .. } if rule == "no-verify")));
}

#[test]
fn test_reset_hard_blocked() {
    let config = Config::default();
    let results = evaluate_command_rules("git reset --hard", &config);
    assert!(results
        .iter()
        .any(|r| matches!(r, RuleResult::Block { rule, .. } if rule == "destructive-ops")));
}

#[test]
fn test_checkout_dot_blocked() {
    let config = Config::default();
    let results = evaluate_command_rules("git checkout .", &config);
    assert!(results
        .iter()
        .any(|r| matches!(r, RuleResult::Block { rule, .. } if rule == "destructive-ops")));
}

#[test]
fn test_clean_f_blocked() {
    let config = Config::default();
    let results = evaluate_command_rules("git clean -f", &config);
    assert!(results
        .iter()
        .any(|r| matches!(r, RuleResult::Block { rule, .. } if rule == "destructive-ops")));
}

#[test]
fn test_restore_dot_blocked() {
    let config = Config::default();
    let results = evaluate_command_rules("git restore .", &config);
    assert!(results
        .iter()
        .any(|r| matches!(r, RuleResult::Block { rule, .. } if rule == "destructive-ops")));
}

#[test]
fn test_normal_push_allowed() {
    let config = Config::default();
    let results = evaluate_command_rules("git push origin main", &config);
    assert!(results
        .iter()
        .all(|r| !matches!(r, RuleResult::Block { .. })));
}

#[test]
fn test_rule_disabled_by_config() {
    let toml = r#"
[rules]
force-push = "off"
"#;
    let config = Config::from_toml_str(toml).unwrap();
    let results = evaluate_command_rules("git push --force origin main", &config);
    assert!(!results
        .iter()
        .any(|r| matches!(r, RuleResult::Block { rule, .. } if rule == "force-push")));
}

#[test]
fn test_chained_commands_checked() {
    let config = Config::default();
    let results = evaluate_command_rules("npm test && git push --force origin main", &config);
    assert!(results
        .iter()
        .any(|r| matches!(r, RuleResult::Block { rule, .. } if rule == "force-push")));
}
