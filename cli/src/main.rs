use clap::{Parser, Subcommand};

use tinstar::commands;

#[derive(Parser)]
#[command(name = "tinstar", version, about = "Git law enforcement for Claude Code")]
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
    }
}
