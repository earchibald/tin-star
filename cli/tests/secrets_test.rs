use tinstar::config::Config;
use tinstar::rules::secrets::Secrets;

#[test]
fn test_detects_aws_key() {
    let secrets = Secrets::new(&Config::default());
    let diff = "+AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE\n";
    let findings = secrets.scan_diff(diff);
    assert!(!findings.is_empty());
}

#[test]
fn test_detects_private_key() {
    let secrets = Secrets::new(&Config::default());
    let diff = "+-----BEGIN RSA PRIVATE KEY-----\n";
    let findings = secrets.scan_diff(diff);
    assert!(!findings.is_empty());
}

#[test]
fn test_detects_generic_api_key() {
    let secrets = Secrets::new(&Config::default());
    let diff = "+api_key = 'sk-1234567890abcdef'\n";
    let findings = secrets.scan_diff(diff);
    assert!(!findings.is_empty());
}

#[test]
fn test_ignores_removed_lines() {
    let secrets = Secrets::new(&Config::default());
    let diff = "-api_key = 'sk-1234567890abcdef'\n";
    let findings = secrets.scan_diff(diff);
    assert!(findings.is_empty());
}

#[test]
fn test_custom_pattern() {
    let toml = r#"
[secrets]
extra-patterns = ["ghp_[A-Za-z0-9]{36}"]
"#;
    let config = Config::from_toml_str(toml).unwrap();
    let secrets = Secrets::new(&config);
    let diff = "+GITHUB_TOKEN=ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghij\n";
    let findings = secrets.scan_diff(diff);
    assert!(!findings.is_empty());
}

#[test]
fn test_ignore_pattern_suppresses() {
    let toml = r#"
[secrets.ignore]
patterns = ["EXAMPLE"]
"#;
    let config = Config::from_toml_str(toml).unwrap();
    let secrets = Secrets::new(&config);
    let diff = "+api_key = 'EXAMPLE_KEY_12345'\n";
    let findings = secrets.scan_diff(diff);
    assert!(findings.is_empty());
}
