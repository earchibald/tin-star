mod force_push;
mod no_verify;
mod destructive_ops;
pub mod commit_to_main;
pub mod commit_message;
pub mod secrets;
pub mod branch_divergence;
pub mod stale_branches;

use std::path::Path;

use crate::config::{Config, Severity};
use crate::git;

// ---------------------------------------------------------------------------
// Rule trait
// ---------------------------------------------------------------------------

/// A command-matching rule. Pure function: given a single command string,
/// returns an optional (rule_name, reason) if the rule fires.
pub trait Rule {
    /// Machine-readable rule name (e.g. "force-push").
    fn name(&self) -> &str;

    /// Check a single command string. Returns `Some((rule_name, reason))` if
    /// the rule fires, `None` if the command is clean.
    fn check_command(&self, command: &str) -> Option<(String, String)>;
}

// ---------------------------------------------------------------------------
// RuleResult
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleResult {
    Allow,
    Warn {
        rule: String,
        reason: String,
    },
    Block {
        rule: String,
        reason: String,
    },
}

// ---------------------------------------------------------------------------
// Command splitting
// ---------------------------------------------------------------------------

/// Naive best-effort split on `&&`, `||`, `;`.
fn split_commands(command: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut rest = command;

    while !rest.is_empty() {
        // Find the earliest delimiter
        let mut earliest: Option<(usize, usize)> = None; // (position, delimiter length)

        for delim in &["&&", "||", ";"] {
            if let Some(pos) = rest.find(delim) {
                if earliest.is_none() || pos < earliest.unwrap().0 {
                    earliest = Some((pos, delim.len()));
                }
            }
        }

        match earliest {
            Some((pos, len)) => {
                let segment = rest[..pos].trim();
                if !segment.is_empty() {
                    parts.push(segment);
                }
                rest = &rest[pos + len..];
            }
            None => {
                let segment = rest.trim();
                if !segment.is_empty() {
                    parts.push(segment);
                }
                break;
            }
        }
    }

    parts
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns true if any segment of the (possibly chained) command starts with `git commit`.
fn is_commit_command(command: &str) -> bool {
    let segments = split_commands(command);
    segments.iter().any(|seg| {
        let args: Vec<&str> = seg.split_whitespace().collect();
        args.len() >= 2 && args[0] == "git" && args[1] == "commit"
    })
}

// ---------------------------------------------------------------------------
// Rule registry
// ---------------------------------------------------------------------------

fn command_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(force_push::ForcePushRule),
        Box::new(no_verify::NoVerifyRule),
        Box::new(destructive_ops::DestructiveOpsRule),
        Box::new(commit_message::CommitMessage),
    ]
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Evaluate all command-matching rules against the given command string.
/// Splits chained commands (`&&`, `||`, `;`) and checks each segment.
/// Respects per-rule severity from the config.
pub fn evaluate_command_rules(command: &str, config: &Config) -> Vec<RuleResult> {
    let segments = split_commands(command);
    let rules = command_rules();
    let mut results = Vec::new();

    for segment in &segments {
        for rule in &rules {
            let severity = config.rule_severity(rule.name());
            if severity == Severity::Off {
                continue;
            }

            if let Some((rule_name, reason)) = rule.check_command(segment) {
                match severity {
                    Severity::Block => results.push(RuleResult::Block {
                        rule: rule_name,
                        reason,
                    }),
                    Severity::Warn => results.push(RuleResult::Warn {
                        rule: rule_name,
                        reason,
                    }),
                    Severity::Off => unreachable!(),
                }
            }
        }
    }

    if results.is_empty() {
        results.push(RuleResult::Allow);
    }

    results
}

/// Evaluate all rules — command-matching and state-matching — against the
/// given command string. For `git commit` commands, also runs state/content
/// rules that require repository context (branch check, secrets scan, etc.).
pub fn evaluate_all_rules(command: &str, config: &Config, project_dir: &Path) -> Vec<RuleResult> {
    let mut results = evaluate_command_rules(command, config);

    // Remove the trailing Allow if we're about to add state results
    if is_commit_command(command) {
        // Remove Allow entries — we'll re-add at the end if needed
        results.retain(|r| !matches!(r, RuleResult::Allow));

        // commit-to-main: check current branch
        let commit_to_main_severity = config.rule_severity("commit-to-main");
        if commit_to_main_severity != Severity::Off {
            if let Ok(branch) = git::current_branch(project_dir) {
                let rule = commit_to_main::CommitToMain;
                if let Some((_name, reason)) = rule.check_with_branch(command, &branch) {
                    match commit_to_main_severity {
                        Severity::Block => results.push(RuleResult::Block {
                            rule: "commit-to-main".into(),
                            reason,
                        }),
                        Severity::Warn => results.push(RuleResult::Warn {
                            rule: "commit-to-main".into(),
                            reason,
                        }),
                        Severity::Off => unreachable!(),
                    }
                }
            }
        }

        // secrets: scan staged diff
        let secrets_severity = config.rule_severity("secrets");
        if secrets_severity != Severity::Off {
            if let Ok(diff) = git::staged_diff(project_dir) {
                let scanner = secrets::Secrets::new(config);
                let findings = scanner.scan_diff(&diff);
                for finding in findings {
                    match secrets_severity {
                        Severity::Block => results.push(RuleResult::Block {
                            rule: "secrets".into(),
                            reason: finding,
                        }),
                        Severity::Warn => results.push(RuleResult::Warn {
                            rule: "secrets".into(),
                            reason: finding,
                        }),
                        Severity::Off => unreachable!(),
                    }
                }
            }
        }

        // branch-divergence
        if let Some(result) = branch_divergence::check(
            project_dir,
            config.rule_severity("branch-divergence"),
            0, // threshold: warn if behind at all
        ) {
            results.push(result);
        }

        // If nothing fired, emit Allow
        if results.is_empty() {
            results.push(RuleResult::Allow);
        }
    }

    results
}
