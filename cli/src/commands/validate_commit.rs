//! Validate-commit-msg command — checks commit message quality.

use std::process;

use serde::Serialize;

use crate::output::print_json;

#[derive(Serialize)]
struct ValidateResult {
    valid: bool,
    reason: String,
}

/// Run the `validate-commit-msg` command.
///
/// Exit codes: 0 = valid, 2 = invalid.
pub fn run(message: &str, json: bool) {
    let trimmed = message.trim();

    let (valid, reason) = if trimmed.is_empty() {
        (false, "commit message is empty".to_string())
    } else if trimmed.len() < 3 {
        (false, format!("commit message too short ({} chars, minimum 3)", trimmed.len()))
    } else {
        (true, String::new())
    };

    let output = ValidateResult {
        valid,
        reason: reason.clone(),
    };

    if json {
        print_json(&output);
    } else if !valid {
        eprintln!("tinstar: invalid commit message: {reason}");
    }

    if !valid {
        process::exit(2);
    }
}
