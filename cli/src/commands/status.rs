use std::path::Path;
use std::process;

use serde::Serialize;

use crate::config::{Config, Severity};
use crate::git;
use crate::output::print_json;

/// Known rule names in display order.
const RULE_NAMES: &[&str] = &[
    "force-push",
    "no-verify",
    "destructive-ops",
    "commit-to-main",
    "secrets",
    "commit-message",
    "branch-divergence",
    "stale-branches",
];

#[derive(Serialize)]
struct StatusOutput {
    branch: String,
    rules: Vec<RuleStatus>,
    warnings: Vec<String>,
}

#[derive(Serialize)]
struct RuleStatus {
    name: String,
    severity: String,
}

fn severity_str(s: Severity) -> &'static str {
    match s {
        Severity::Block => "block",
        Severity::Warn => "warn",
        Severity::Off => "off",
    }
}

/// Run the `status` command.
///
/// Shows active rules with severities, current branch, and any warnings.
pub fn run(json: bool, project_dir: &Path) {
    let config = match Config::load(project_dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("tinstar: failed to load config: {e}");
            process::exit(1);
        }
    };

    let branch = git::current_branch(project_dir).unwrap_or_else(|_| "unknown".into());

    let rules: Vec<RuleStatus> = RULE_NAMES
        .iter()
        .map(|name| {
            let sev = config.rule_severity(name);
            RuleStatus {
                name: name.to_string(),
                severity: severity_str(sev).to_string(),
            }
        })
        .collect();

    let mut warnings = Vec::new();

    if git::is_detached_head(project_dir) {
        warnings.push("HEAD is detached".into());
    }

    if git::is_dirty(project_dir) {
        warnings.push("Working tree has uncommitted changes".into());
    }

    let output = StatusOutput {
        branch,
        rules,
        warnings,
    };

    if json {
        print_json(&output);
    } else {
        println!("tinstar v{}", env!("CARGO_PKG_VERSION"));
        println!("Branch: {}", output.branch);
        println!();
        println!("Rules:");
        for rule in &output.rules {
            let marker = match rule.severity.as_str() {
                "block" => "X",
                "warn" => "!",
                _ => "-",
            };
            println!("  [{marker}] {}: {}", rule.name, rule.severity);
        }
        if !output.warnings.is_empty() {
            println!();
            println!("Warnings:");
            for w in &output.warnings {
                println!("  - {w}");
            }
        }
    }
}
