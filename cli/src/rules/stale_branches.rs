use std::path::Path;

use chrono::NaiveDateTime;

use super::RuleResult;
use crate::config::Severity;
use crate::git;

/// Checks for local branches whose last commit is older than `stale_days`.
pub fn check(dir: &Path, severity: Severity, stale_days: u32) -> Vec<RuleResult> {
    if severity == Severity::Off {
        return Vec::new();
    }

    let branches = match git::list_branches(dir) {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };

    let now = chrono::Utc::now().naive_utc();
    let threshold = chrono::Duration::days(stale_days as i64);
    let mut results = Vec::new();

    for branch in &branches {
        if let Some(ref date_str) = branch.last_commit_date {
            // git for-each-ref returns ISO 8601 like "2024-01-15 10:30:00 -0500"
            // Try parsing with timezone offset stripped (NaiveDateTime)
            let clean = date_str.trim();
            // Strip timezone offset (last 6 chars like " -0500")
            let without_tz = if clean.len() > 6 {
                &clean[..clean.len() - 6]
            } else {
                clean
            };

            if let Ok(commit_time) = NaiveDateTime::parse_from_str(without_tz, "%Y-%m-%d %H:%M:%S")
            {
                let age = now - commit_time;
                if age > threshold {
                    let reason = format!(
                        "Branch '{}' has not been committed to in {} days (threshold: {}).",
                        branch.name,
                        age.num_days(),
                        stale_days
                    );
                    let result = match severity {
                        Severity::Block => RuleResult::Block {
                            rule: "stale-branches".into(),
                            reason,
                        },
                        Severity::Warn => RuleResult::Warn {
                            rule: "stale-branches".into(),
                            reason,
                        },
                        Severity::Off => unreachable!(),
                    };
                    results.push(result);
                }
            }
        }
    }

    results
}
