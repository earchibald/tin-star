//! Scan-diff command — scan staged changes for secrets.

use std::path::Path;
use std::process;

use serde::Serialize;

use crate::config::Config;
use crate::git;
use crate::output::print_json;
use crate::rules::secrets::Secrets;

#[derive(Serialize)]
struct ScanDiffOutput {
    findings: Vec<String>,
    clean: bool,
}

/// Run the `scan-diff` command.
pub fn run(project_dir: &Path, json: bool) {
    let config = match Config::load(project_dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("tinstar: failed to load config: {e}");
            process::exit(1);
        }
    };

    let diff = match git::staged_diff(project_dir) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("tinstar: failed to get staged diff: {e}");
            process::exit(1);
        }
    };

    let scanner = Secrets::new(&config);
    let findings = scanner.scan_diff(&diff);

    let output = ScanDiffOutput {
        clean: findings.is_empty(),
        findings: findings.clone(),
    };

    if json {
        print_json(&output);
    } else {
        if findings.is_empty() {
            println!("No secrets detected in staged changes.");
        } else {
            println!("tinstar: {} potential secret(s) found:", findings.len());
            for f in &findings {
                println!("  ! {f}");
            }
        }
    }

    if !findings.is_empty() {
        process::exit(2);
    }
}
