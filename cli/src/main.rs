use clap::{Parser, Subcommand};

use tinstar::commands;

#[derive(Parser)]
#[command(
    name = "tinstar",
    version,
    about = "Git law enforcement for Claude Code"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run applicable rules against a git command
    Check {
        /// The git command to check
        #[arg(long)]
        command: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Lightweight git state check (used by Stop hook)
    CheckState {
        #[arg(long)]
        json: bool,
    },
    /// Show active rules and recent enforcement
    Status {
        #[arg(long)]
        json: bool,
    },
    /// Manage leak records (rule violations)
    Issue {
        #[command(subcommand)]
        action: IssueAction,
    },
    /// Manage project configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Branch hygiene sweep
    Sweep {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Scan staged diff for secrets
    ScanDiff {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Validate a commit message
    ValidateCommitMsg {
        /// The commit message to validate
        #[arg(long)]
        message: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum IssueAction {
    /// Record a new leak
    Create {
        #[arg(long)]
        rule: String,
        #[arg(long)]
        command: String,
        #[arg(long)]
        hook: String,
        #[arg(long)]
        project: String,
        #[arg(long)]
        branch: String,
        #[arg(long)]
        dirty: bool,
    },
    /// List recorded leaks
    List {
        #[arg(long)]
        json: bool,
    },
    /// Create GitHub issue(s) from leaks
    Report {
        /// Report all leaks (not just the latest)
        #[arg(long)]
        all: bool,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show {
        #[arg(long)]
        json: bool,
    },
    /// Set a rule severity
    Set {
        /// Rule name (e.g. force-push)
        rule: String,
        /// Severity level (block, warn, off)
        severity: String,
    },
    /// Reset to defaults (delete .tinstar.toml)
    Reset,
}

fn main() {
    let cli = Cli::parse();
    let project_dir = std::env::current_dir().unwrap_or_else(|e| {
        eprintln!("tinstar: cannot determine current directory: {e}");
        std::process::exit(1);
    });

    match cli.command {
        Commands::Check { command, json } => {
            commands::check::run(&command, json, &project_dir);
        }
        Commands::CheckState { json } => {
            commands::check_state::run(json, &project_dir);
        }
        Commands::Status { json } => {
            commands::status::run(json, &project_dir);
        }
        Commands::Issue { action } => match action {
            IssueAction::Create {
                rule,
                command,
                hook,
                project,
                branch,
                dirty,
            } => {
                commands::issue::run_create(&rule, &command, &hook, &project, &branch, dirty);
            }
            IssueAction::List { json } => {
                commands::issue::run_list(json);
            }
            IssueAction::Report { all } => {
                commands::issue::run_report(all);
            }
        },
        Commands::Config { action } => match action {
            ConfigAction::Show { json } => {
                commands::config_cmd::run_show(&project_dir, json);
            }
            ConfigAction::Set { rule, severity } => {
                commands::config_cmd::run_set(&project_dir, &rule, &severity);
            }
            ConfigAction::Reset => {
                commands::config_cmd::run_reset(&project_dir);
            }
        },
        Commands::Sweep { json } => {
            commands::sweep::run(&project_dir, json);
        }
        Commands::ScanDiff { json } => {
            commands::scan_diff::run(&project_dir, json);
        }
        Commands::ValidateCommitMsg { message, json } => {
            commands::validate_commit::run(&message, json);
        }
    }
}
