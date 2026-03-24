use clap::{Parser, Subcommand};

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
    match cli.command {
        Commands::Check { command, json } => {
            eprintln!("check not yet implemented: {command}");
            std::process::exit(1);
        }
        Commands::CheckState { json } => {
            eprintln!("check-state not yet implemented");
            std::process::exit(1);
        }
        Commands::Status { json } => {
            println!("tinstar v{}", env!("CARGO_PKG_VERSION"));
            println!("No rules implemented yet.");
        }
    }
}
