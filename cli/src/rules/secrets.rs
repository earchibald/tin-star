use regex::Regex;

use crate::config::Config;

/// Built-in secret detection patterns.
const BUILTIN_PATTERNS: &[&str] = &[
    r"AKIA[0-9A-Z]{16}",
    r"-----BEGIN.*PRIVATE KEY-----",
    r#"(?i)api[_-]?key\s*[:=]\s*['"][^'""]+"#,
    r#"(?i)password\s*[:=]\s*['"][^'""]+"#,
];

pub struct Secrets {
    patterns: Vec<Regex>,
    ignore_patterns: Vec<String>,
}

impl Secrets {
    pub fn new(config: &Config) -> Self {
        let mut patterns = Vec::new();

        for pat in BUILTIN_PATTERNS {
            if let Ok(re) = Regex::new(pat) {
                patterns.push(re);
            }
        }

        for pat in &config.secrets.extra_patterns {
            if let Ok(re) = Regex::new(pat) {
                patterns.push(re);
            }
        }

        Self {
            patterns,
            ignore_patterns: config.secrets.ignore_patterns.clone(),
        }
    }

    /// Scan a unified diff string for secrets in added lines.
    /// Returns a list of findings (human-readable descriptions).
    pub fn scan_diff(&self, diff: &str) -> Vec<String> {
        let mut findings = Vec::new();

        for line in diff.lines() {
            // Only scan added lines (starting with '+'), but not diff headers ('+++')
            if !line.starts_with('+') || line.starts_with("+++") {
                continue;
            }

            let content = &line[1..]; // strip the leading '+'

            // Check ignore patterns — if any ignore pattern matches, skip the line
            if self
                .ignore_patterns
                .iter()
                .any(|ip| content.contains(ip))
            {
                continue;
            }

            for re in &self.patterns {
                if re.is_match(content) {
                    findings.push(format!(
                        "Possible secret detected (pattern: {}): {}",
                        re.as_str(),
                        content.trim()
                    ));
                    break; // one finding per line is enough
                }
            }
        }

        findings
    }
}
