use std::collections::HashMap;
use std::path::Path;

use glob::Pattern;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Block,
    Warn,
    Off,
}

// ---------------------------------------------------------------------------
// Known rules with their default severities
// ---------------------------------------------------------------------------

const KNOWN_RULES: &[(&str, Severity)] = &[
    ("force-push", Severity::Block),
    ("no-verify", Severity::Block),
    ("destructive-ops", Severity::Block),
    ("commit-to-main", Severity::Warn),
    ("secrets", Severity::Block),
    ("commit-message", Severity::Block),
    ("branch-divergence", Severity::Warn),
    ("stale-branches", Severity::Warn),
];

fn default_rules() -> HashMap<String, Severity> {
    KNOWN_RULES
        .iter()
        .map(|(name, sev)| (name.to_string(), *sev))
        .collect()
}

fn is_known_rule(name: &str) -> bool {
    KNOWN_RULES.iter().any(|(k, _)| *k == name)
}

// ---------------------------------------------------------------------------
// Config — the primary public interface
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Config {
    rules: HashMap<String, Severity>,
    pub branches: BranchesConfig,
    pub secrets: SecretsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rules: default_rules(),
            branches: BranchesConfig::default(),
            secrets: SecretsConfig::default(),
        }
    }
}

impl Config {
    /// Parse a TOML string into a `Config`, merging with defaults.
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        let raw: RawConfig = toml::from_str(s)?;
        let mut config = Self::default();

        // Merge rule overrides — only apply to known rules.
        if let Some(rules) = raw.rules {
            for (name, sev_str) in &rules {
                if is_known_rule(name) {
                    if let Some(sev) = parse_severity(sev_str) {
                        config.rules.insert(name.clone(), sev);
                    }
                }
                // Unknown rules are silently ignored (forward-compatible).
            }
        }

        // Branches
        if let Some(b) = raw.branches {
            if let Some(protected) = b.protected {
                config.branches.protected = protected;
            }
            if let Some(naming) = b.naming {
                config.branches.naming = Some(naming);
            }
            if let Some(stale_days) = b.stale_days {
                config.branches.stale_days = stale_days;
            }
        }

        // Secrets
        if let Some(s) = raw.secrets {
            if let Some(ignore) = s.ignore {
                if let Some(files) = ignore.files {
                    config.secrets.ignore_files = files;
                }
            }
            if let Some(extra) = s.extra_patterns {
                config.secrets.extra_patterns = extra;
            }
        }

        Ok(config)
    }

    /// Load config from `.tinstar.toml` in the given directory.
    /// Returns defaults if the file does not exist.
    pub fn load(project_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let path = project_dir.join(".tinstar.toml");
        if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            Ok(Self::from_toml_str(&contents)?)
        } else {
            Ok(Self::default())
        }
    }

    /// Look up the effective severity for a rule by name.
    /// Returns `Off` for unknown rules.
    pub fn rule_severity(&self, name: &str) -> Severity {
        if !is_known_rule(name) {
            return Severity::Off;
        }
        self.rules.get(name).copied().unwrap_or(Severity::Off)
    }
}

// ---------------------------------------------------------------------------
// BranchesConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BranchesConfig {
    pub protected: Vec<String>,
    pub naming: Option<String>,
    pub stale_days: u32,
}

impl Default for BranchesConfig {
    fn default() -> Self {
        Self {
            protected: vec!["main".into(), "master".into()],
            naming: None,
            stale_days: 30,
        }
    }
}

impl BranchesConfig {
    /// Check whether a branch name matches any protected pattern.
    /// Supports glob patterns (e.g. `release/*`).
    pub fn is_protected(&self, branch: &str) -> bool {
        self.protected.iter().any(|pat| {
            if let Ok(p) = Pattern::new(pat) {
                p.matches(branch)
            } else {
                pat == branch
            }
        })
    }
}

// ---------------------------------------------------------------------------
// SecretsConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SecretsConfig {
    pub ignore_files: Vec<String>,
    pub extra_patterns: Vec<String>,
}

impl Default for SecretsConfig {
    fn default() -> Self {
        Self {
            ignore_files: Vec::new(),
            extra_patterns: Vec::new(),
        }
    }
}

impl SecretsConfig {
    /// Check whether a file path should be excluded from secret scanning.
    pub fn should_ignore_file(&self, path: &str) -> bool {
        self.ignore_files.iter().any(|pat| {
            if let Ok(p) = Pattern::new(pat) {
                p.matches(path)
            } else {
                pat == path
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Raw TOML deserialization structs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct RawConfig {
    #[allow(dead_code)]
    version: Option<u32>,
    rules: Option<HashMap<String, String>>,
    branches: Option<RawBranches>,
    secrets: Option<RawSecrets>,
}

#[derive(Deserialize)]
struct RawBranches {
    protected: Option<Vec<String>>,
    naming: Option<String>,
    #[serde(rename = "stale-days")]
    stale_days: Option<u32>,
}

#[derive(Deserialize)]
struct RawSecrets {
    ignore: Option<RawSecretsIgnore>,
    extra_patterns: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct RawSecretsIgnore {
    files: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_severity(s: &str) -> Option<Severity> {
    match s {
        "block" => Some(Severity::Block),
        "warn" => Some(Severity::Warn),
        "off" => Some(Severity::Off),
        _ => None,
    }
}
