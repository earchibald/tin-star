use std::path::Path;
use std::process;

use crate::config::Config;
use crate::git;
use crate::output::{print_json, CheckStateResult, StateIssue};

/// Run the `check-state` command.
///
/// Performs lightweight git state checks (detached HEAD, dirty working tree,
/// dirty state on a protected branch).
/// Exit codes: 0 = clean, 1 = internal error, 2 = issues found.
pub fn run(json: bool, project_dir: &Path) {
    let config = match Config::load(project_dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("tinstar: failed to load config: {e}");
            process::exit(1);
        }
    };

    let mut issues = Vec::new();

    // Check detached HEAD
    if git::is_detached_head(project_dir) {
        issues.push(StateIssue {
            severity: "warn".into(),
            message: "HEAD is detached — not on any branch".into(),
        });
    }

    // Check dirty working tree
    let dirty = git::is_dirty(project_dir);
    if dirty {
        issues.push(StateIssue {
            severity: "warn".into(),
            message: "Working tree has uncommitted changes".into(),
        });
    }

    // Check if on a protected branch with dirty state
    if dirty {
        if let Ok(branch) = git::current_branch(project_dir) {
            if config.branches.is_protected(&branch) {
                issues.push(StateIssue {
                    severity: "block".into(),
                    message: format!(
                        "Dirty working tree on protected branch '{branch}'"
                    ),
                });
            }
        }
    }

    let result = CheckStateResult {
        issues: issues.clone(),
    };

    if json {
        print_json(&result);
    } else {
        if issues.is_empty() {
            println!("tinstar: repository state is clean");
        } else {
            for issue in &issues {
                eprintln!(
                    "tinstar: [{}] {}",
                    issue.severity.to_uppercase(),
                    issue.message
                );
            }
        }
    }

    if issues.iter().any(|i| i.severity == "block") {
        process::exit(2);
    }
}
