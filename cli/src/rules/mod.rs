mod force_push;
mod no_verify;
mod destructive_ops;

use crate::config::{Config, Severity};

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
// Rule registry
// ---------------------------------------------------------------------------

fn command_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(force_push::ForcePushRule),
        Box::new(no_verify::NoVerifyRule),
        Box::new(destructive_ops::DestructiveOpsRule),
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
