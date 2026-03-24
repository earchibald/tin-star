//! Config management commands — show, set, reset.

use std::path::Path;
use std::process;

use crate::config::{Config, Severity};

/// Known rule names for the annotated template.
const RULE_NAMES: &[(&str, &str)] = &[
    ("force-push", "block"),
    ("no-verify", "block"),
    ("destructive-ops", "block"),
    ("commit-to-main", "warn"),
    ("secrets", "block"),
    ("commit-message", "block"),
    ("branch-divergence", "warn"),
    ("stale-branches", "warn"),
];

/// `tinstar config show` — print current effective config.
pub fn run_show(project_dir: &Path, json: bool) {
    let config = match Config::load(project_dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("tinstar: failed to load config: {e}");
            process::exit(1);
        }
    };

    if json {
        let mut rules = serde_json::Map::new();
        for (name, _default) in RULE_NAMES {
            let sev = config.rule_severity(name);
            rules.insert(
                name.to_string(),
                serde_json::Value::String(severity_str(sev).into()),
            );
        }
        let out = serde_json::json!({ "rules": rules });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        let path = project_dir.join(".tinstar.toml");
        if path.exists() {
            println!("Config: {}", path.display());
        } else {
            println!("Config: defaults (no .tinstar.toml)");
        }
        println!();
        println!("Rules:");
        for (name, _default) in RULE_NAMES {
            let sev = config.rule_severity(name);
            let marker = match sev {
                Severity::Block => "X",
                Severity::Warn => "!",
                Severity::Off => "-",
            };
            println!("  [{marker}] {name}: {}", severity_str(sev));
        }
    }
}

/// `tinstar config set <rule> <severity>` — create or update `.tinstar.toml`.
pub fn run_set(project_dir: &Path, rule: &str, severity: &str) {
    // Validate severity
    if !["block", "warn", "off"].contains(&severity) {
        eprintln!("tinstar: invalid severity '{severity}' (must be block, warn, or off)");
        process::exit(1);
    }

    // Validate rule name
    if !RULE_NAMES.iter().any(|(name, _)| *name == rule) {
        eprintln!("tinstar: unknown rule '{rule}'");
        process::exit(1);
    }

    let path = project_dir.join(".tinstar.toml");

    if path.exists() {
        // Update existing file — read, parse, modify, write
        let contents = std::fs::read_to_string(&path).unwrap_or_default();
        let new_contents = update_rule_in_toml(&contents, rule, severity);
        std::fs::write(&path, new_contents).unwrap_or_else(|e| {
            eprintln!("tinstar: failed to write config: {e}");
            process::exit(1);
        });
    } else {
        // Create new annotated template with the changed rule
        let template = generate_template(rule, severity);
        std::fs::write(&path, template).unwrap_or_else(|e| {
            eprintln!("tinstar: failed to write config: {e}");
            process::exit(1);
        });
    }

    println!("tinstar: set {rule} = {severity} in {}", path.display());
}

/// `tinstar config reset` — delete `.tinstar.toml`.
pub fn run_reset(project_dir: &Path) {
    let path = project_dir.join(".tinstar.toml");
    if path.exists() {
        std::fs::remove_file(&path).unwrap_or_else(|e| {
            eprintln!("tinstar: failed to remove config: {e}");
            process::exit(1);
        });
        println!("tinstar: removed {}", path.display());
    } else {
        println!("tinstar: no .tinstar.toml to remove (already using defaults)");
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn severity_str(s: Severity) -> &'static str {
    match s {
        Severity::Block => "block",
        Severity::Warn => "warn",
        Severity::Off => "off",
    }
}

/// Generate a full annotated template with all defaults, overriding one rule.
fn generate_template(changed_rule: &str, changed_severity: &str) -> String {
    let mut out = String::new();
    out.push_str("# tin-star configuration\n");
    out.push_str("# Severity levels: block | warn | off\n");
    out.push_str("version = 1\n\n");
    out.push_str("[rules]\n");

    for (name, default) in RULE_NAMES {
        let sev = if *name == changed_rule {
            changed_severity
        } else {
            default
        };
        out.push_str(&format!("{name} = \"{sev}\"\n"));
    }

    out.push_str("\n[branches]\n");
    out.push_str("protected = [\"main\", \"master\"]\n");
    out.push_str("# naming = \"^(feat|fix|chore|docs|test|ci)/\"\n");
    out.push_str("stale-days = 30\n");
    out.push_str("\n[secrets]\n");
    out.push_str("# extra-patterns = []\n");
    out.push_str("# [secrets.ignore]\n");
    out.push_str("# files = []\n");
    out.push_str("# patterns = []\n");

    out
}

/// Update a rule value in existing TOML text. Simple line-based replacement.
fn update_rule_in_toml(contents: &str, rule: &str, severity: &str) -> String {
    let target = format!("{rule} = ");
    let mut found = false;
    let mut lines: Vec<String> = Vec::new();

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&target) || trimmed.starts_with(&format!("{rule}=")) {
            lines.push(format!("{rule} = \"{severity}\""));
            found = true;
        } else {
            lines.push(line.to_string());
        }
    }

    // If the rule wasn't found, add it under [rules]
    if !found {
        let mut result = Vec::new();
        let mut added = false;
        for line in &lines {
            result.push(line.clone());
            if !added && line.trim() == "[rules]" {
                result.push(format!("{rule} = \"{severity}\""));
                added = true;
            }
        }
        if !added {
            // No [rules] section — append one
            result.push(String::new());
            result.push("[rules]".to_string());
            result.push(format!("{rule} = \"{severity}\""));
        }
        lines = result;
    }

    let mut out = lines.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}
