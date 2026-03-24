use std::path::Path;

use crate::config::Severity;
use crate::git;
use super::RuleResult;

/// Checks whether the current branch has fallen behind its upstream.
pub fn check(dir: &Path, severity: Severity, threshold: usize) -> Option<RuleResult> {
    if severity == Severity::Off {
        return None;
    }

    let behind = match git::commits_behind_upstream(dir) {
        Ok(Some(n)) => n,
        _ => return None, // no upstream or error — nothing to warn about
    };

    if behind <= threshold {
        return None;
    }

    let reason = format!(
        "Current branch is {} commits behind upstream. Consider pulling.",
        behind
    );

    match severity {
        Severity::Block => Some(RuleResult::Block {
            rule: "branch-divergence".into(),
            reason,
        }),
        Severity::Warn => Some(RuleResult::Warn {
            rule: "branch-divergence".into(),
            reason,
        }),
        Severity::Off => None,
    }
}
