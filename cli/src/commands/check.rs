use std::path::Path;
use std::process;

use crate::config::Config;
use crate::output::{print_json, CheckResult};
use crate::rules::{evaluate_all_rules, RuleResult};

/// Run the `check` command.
///
/// Evaluates all rules against the given command string.
/// Exit codes: 0 = allow, 1 = internal error, 2 = block.
pub fn run(command: &str, json: bool, project_dir: &Path) {
    let config = match Config::load(project_dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("tinstar: failed to load config: {e}");
            process::exit(1);
        }
    };

    let results = evaluate_all_rules(command, &config, project_dir);

    let mut has_block = false;
    let mut block_rule = String::new();
    let mut block_reason = String::new();

    for result in &results {
        match result {
            RuleResult::Block { rule, reason } => {
                if !has_block {
                    block_rule = rule.clone();
                    block_reason = reason.clone();
                }
                has_block = true;
                eprintln!("tinstar: BLOCKED by {rule}: {reason}");
            }
            RuleResult::Warn { rule, reason } => {
                eprintln!("tinstar: WARNING {rule}: {reason}");
            }
            RuleResult::Allow => {}
        }
    }

    if has_block {
        if json {
            print_json(&CheckResult {
                decision: "block".into(),
                rule: block_rule,
                reason: block_reason,
            });
        }
        process::exit(2);
    }

    if json {
        print_json(&CheckResult {
            decision: "allow".into(),
            rule: String::new(),
            reason: String::new(),
        });
    }
}
