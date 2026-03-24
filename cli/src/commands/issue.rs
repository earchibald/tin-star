//! Issue tracking — records rule violations ("leaks") to disk.

use std::path::Path;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leak {
    pub id: String,
    pub timestamp: String,
    pub rule: String,
    pub command: String,
    pub hook: String,
    pub project: String,
    pub git_state: GitState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitState {
    pub branch: String,
    pub dirty: bool,
}

// ---------------------------------------------------------------------------
// Public API (library functions, testable without CLI)
// ---------------------------------------------------------------------------

/// Create a new leak record on disk. Creates the directory if needed.
pub fn create_leak(
    leaks_dir: &Path,
    rule: &str,
    command: &str,
    hook: &str,
    project: &str,
    branch: &str,
    dirty: bool,
) {
    std::fs::create_dir_all(leaks_dir).unwrap_or_else(|e| {
        eprintln!("tinstar: cannot create leaks dir: {e}");
    });

    let id = format!("leak-{}", Uuid::new_v4());
    let leak = Leak {
        id: id.clone(),
        timestamp: Utc::now().to_rfc3339(),
        rule: rule.to_string(),
        command: command.to_string(),
        hook: hook.to_string(),
        project: project.to_string(),
        git_state: GitState {
            branch: branch.to_string(),
            dirty,
        },
    };

    let json = serde_json::to_string_pretty(&leak).expect("serialize leak");
    let path = leaks_dir.join(format!("{id}.json"));
    std::fs::write(&path, json).unwrap_or_else(|e| {
        eprintln!("tinstar: failed to write leak {path:?}: {e}");
    });
}

/// Read all leak files from the given directory.
pub fn list_leaks(leaks_dir: &Path) -> Vec<Leak> {
    let mut leaks = Vec::new();
    let entries = match std::fs::read_dir(leaks_dir) {
        Ok(e) => e,
        Err(_) => return leaks,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(leak) = serde_json::from_str::<Leak>(&contents) {
                    leaks.push(leak);
                }
            }
        }
    }

    leaks
}

// ---------------------------------------------------------------------------
// CLI entry points
// ---------------------------------------------------------------------------

/// `tinstar issue create` handler.
pub fn run_create(rule: &str, command: &str, hook: &str, project: &str, branch: &str, dirty: bool) {
    let leaks_dir = leaks_directory();
    create_leak(&leaks_dir, rule, command, hook, project, branch, dirty);
    eprintln!("tinstar: leak recorded for rule '{rule}'");
}

/// `tinstar issue list` handler.
pub fn run_list(json: bool) {
    let leaks_dir = leaks_directory();
    let leaks = list_leaks(&leaks_dir);

    if json {
        println!("{}", serde_json::to_string(&leaks).unwrap());
    } else {
        if leaks.is_empty() {
            println!("No recorded leaks.");
            return;
        }
        for leak in &leaks {
            println!(
                "[{}] rule={} cmd={} branch={} @ {}",
                leak.id, leak.rule, leak.command, leak.git_state.branch, leak.timestamp,
            );
        }
    }
}

/// `tinstar issue report` handler — creates a GitHub issue via `gh`.
pub fn run_report(all: bool) {
    let leaks_dir = leaks_directory();
    let leaks = list_leaks(&leaks_dir);

    if leaks.is_empty() {
        println!("No leaks to report.");
        return;
    }

    let leaks_to_report = if all {
        leaks.clone()
    } else {
        vec![leaks.last().unwrap().clone()]
    };

    for leak in &leaks_to_report {
        let title = format!("tin-star: {} violation detected", leak.rule);
        let body = format!(
            "**Rule:** {}\n**Command:** `{}`\n**Branch:** {}\n**Hook:** {}\n**Timestamp:** {}\n**Project:** {}",
            leak.rule, leak.command, leak.git_state.branch, leak.hook, leak.timestamp, leak.project,
        );

        let status = std::process::Command::new("gh")
            .args(["issue", "create", "--title", &title, "--body", &body])
            .status();

        match status {
            Ok(s) if s.success() => eprintln!("tinstar: issue created for {}", leak.id),
            Ok(s) => eprintln!("tinstar: gh exited with {s}"),
            Err(e) => eprintln!("tinstar: failed to run gh: {e}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn leaks_directory() -> std::path::PathBuf {
    dirs_next().join("leaks")
}

fn dirs_next() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    std::path::PathBuf::from(home).join(".tinstar")
}
