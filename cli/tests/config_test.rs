#[test]
fn test_default_config_has_all_rules() {
    let config = tinstar::config::Config::default();
    assert_eq!(config.rule_severity("force-push"), tinstar::config::Severity::Block);
    assert_eq!(config.rule_severity("commit-to-main"), tinstar::config::Severity::Warn);
    assert_eq!(config.rule_severity("nonexistent"), tinstar::config::Severity::Off);
}

#[test]
fn test_parse_toml_overrides_defaults() {
    let toml = r#"
version = 1
[rules]
force-push = "warn"
commit-to-main = "block"
"#;
    let config = tinstar::config::Config::from_toml_str(toml).unwrap();
    assert_eq!(config.rule_severity("force-push"), tinstar::config::Severity::Warn);
    assert_eq!(config.rule_severity("commit-to-main"), tinstar::config::Severity::Block);
    // Unmentioned rules keep defaults
    assert_eq!(config.rule_severity("no-verify"), tinstar::config::Severity::Block);
}

#[test]
fn test_unknown_rules_ignored() {
    let toml = r#"
version = 1
[rules]
future-rule = "block"
"#;
    let config = tinstar::config::Config::from_toml_str(toml).unwrap();
    // Unknown rules don't crash, return Off
    assert_eq!(config.rule_severity("future-rule"), tinstar::config::Severity::Off);
}

#[test]
fn test_branches_config() {
    let toml = r#"
[branches]
protected = ["main", "master", "release/*"]
naming = "^(feat|fix)/.*"
stale-days = 21
"#;
    let config = tinstar::config::Config::from_toml_str(toml).unwrap();
    assert_eq!(config.branches.stale_days, 21);
    assert!(config.branches.is_protected("main"));
    assert!(config.branches.is_protected("release/1.0"));
    assert!(!config.branches.is_protected("feat/foo"));
}

#[test]
fn test_secrets_ignore_files_glob() {
    let toml = r#"
[secrets.ignore]
files = ["*.test.ts", "fixtures/**"]
"#;
    let config = tinstar::config::Config::from_toml_str(toml).unwrap();
    assert!(config.secrets.should_ignore_file("foo.test.ts"));
    assert!(config.secrets.should_ignore_file("fixtures/data/keys.json"));
    assert!(!config.secrets.should_ignore_file("src/main.ts"));
}
