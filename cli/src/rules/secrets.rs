use regex::Regex;

use crate::config::Config;

/// Built-in secret detection patterns.
const BUILTIN_PATTERNS: &[&str] = &[
    // AWS
    r"AKIA[0-9A-Z]{16}",
    // PEM private keys  // tinstar:ignore
    r"-----BEGIN.*PRIVATE KEY-----", // tinstar:ignore
    // Generic quoted api_key / password assignments
    r#"(?i)api[_-]?key\s*[:=]\s*['"][^'""]+"#,
    r#"(?i)password\s*[:=]\s*['"][^'""]+"#,
    // GitHub PAT (classic ghp_ and fine-grained github_pat_)
    r"gh[po]_[A-Za-z0-9]{36,255}",
    r"github_pat_[A-Za-z0-9_]{82}",
    // GitLab PAT
    r"glpat-[A-Za-z0-9\-]{20}",
    // Anthropic API key
    r"sk-ant-api0[0-9]-[A-Za-z0-9\-_]{20,}",
    // OpenAI project key
    r"sk-proj-[A-Za-z0-9\-_]{20,}",
    // Google API key
    r"AIza[0-9A-Za-z\-_]{35}",
    // Slack tokens
    r"xox[baprs]-[A-Za-z0-9\-]{10,}",
    // Stripe live secret / restricted keys
    r"sk_live_[0-9a-zA-Z]{24,}",
    r"rk_live_[0-9a-zA-Z]{24,}",
    // JWT (two base64url segments separated by dot)
    r"eyJ[A-Za-z0-9_\-]{10,}\.eyJ[A-Za-z0-9_\-]{10,}",
    // Database connection URLs with embedded credentials
    r"(?i)(postgres|postgresql|mysql|mongodb(\+srv)?|redis)://[^:@\s]+:[^@\s]+@",
    // npm tokens
    r"npm_[A-Za-z0-9]{36}",
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

            // Inline annotation: skip lines explicitly marked safe
            if content.contains("tinstar:ignore") {
                continue;
            }

            // Check ignore patterns — if any ignore pattern matches, skip the line
            if self.ignore_patterns.iter().any(|ip| content.contains(ip)) {
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
