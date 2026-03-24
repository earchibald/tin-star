//! Sweep command — branch hygiene and leak pruning.

use std::path::Path;

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::config::Config;
use crate::git;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct SweepOutput {
    branches: Vec<BranchReport>,
    warnings: Vec<String>,
}

#[derive(Serialize)]
struct BranchReport {
    name: String,
    has_remote: bool,
    last_commit_date: Option<String>,
    issues: Vec<String>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Run the `sweep` command — branch hygiene check.
pub fn run(project_dir: &Path, json: bool) {
    let config = match Config::load(project_dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("tinstar: failed to load config: {e}");
            std::process::exit(1);
        }
    };

    let branches = match git::list_branches(project_dir) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("tinstar: failed to list branches: {e}");
            std::process::exit(1);
        }
    };

    let mut reports = Vec::new();
    let mut warnings = Vec::new();
    let now = Utc::now();

    for branch in &branches {
        let mut issues = Vec::new();

        // Check stale
        if let Some(ref date_str) = branch.last_commit_date {
            if let Ok(date) = DateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S %z") {
                let age_days = (now - date.with_timezone(&Utc)).num_days();
                if age_days > config.branches.stale_days as i64 {
                    issues.push(format!("stale ({age_days} days since last commit)"));
                }
            }
        }

        // Check orphan (no remote tracking)
        if !branch.has_remote && !config.branches.is_protected(&branch.name) {
            issues.push("orphan (no remote tracking branch)".into());
        }

        // Check naming convention
        if let Some(ref pattern) = config.branches.naming {
            if !config.branches.is_protected(&branch.name) {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if !re.is_match(&branch.name) {
                        issues.push(format!("naming: does not match '{pattern}'"));
                    }
                }
            }
        }

        if !issues.is_empty() {
            warnings.push(format!("{}: {}", branch.name, issues.join(", ")));
        }

        reports.push(BranchReport {
            name: branch.name.clone(),
            has_remote: branch.has_remote,
            last_commit_date: branch.last_commit_date.clone(),
            issues,
        });
    }

    let output = SweepOutput {
        branches: reports,
        warnings,
    };

    if json {
        println!("{}", serde_json::to_string(&output).unwrap());
    } else {
        println!("tinstar sweep — {} branches", output.branches.len());
        if output.warnings.is_empty() {
            println!("All branches clean.");
        } else {
            println!();
            for w in &output.warnings {
                println!("  ! {w}");
            }
        }
    }
}

/// Prune leak files older than `max_age_days`. Returns the number pruned.
pub fn prune_old_leaks(leaks_dir: &Path, max_age_days: u64) -> usize {
    let entries = match std::fs::read_dir(leaks_dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    let now = Utc::now();
    let mut pruned = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Parse just the timestamp field
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&contents);
        if let Ok(val) = parsed {
            if let Some(ts) = val.get("timestamp").and_then(|v| v.as_str()) {
                if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
                    let age_days = (now - dt.with_timezone(&Utc)).num_days();
                    if age_days > max_age_days as i64 {
                        if std::fs::remove_file(&path).is_ok() {
                            pruned += 1;
                        }
                    }
                }
            }
        }
    }

    pruned
}
