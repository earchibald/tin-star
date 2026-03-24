use super::Rule;
use crate::git::extract_commit_message;

pub struct CommitMessage;

impl Rule for CommitMessage {
    fn name(&self) -> &str {
        "commit-message"
    }

    fn check_command(&self, command: &str) -> Option<(String, String)> {
        // Only applies to git commit commands
        let args: Vec<&str> = command.split_whitespace().collect();
        if args.len() < 2 || args[0] != "git" || args[1] != "commit" {
            return None;
        }

        // If no -m flag, allow (user will get an editor)
        let msg = extract_commit_message(command)?;

        let trimmed = msg.trim();

        if trimmed.is_empty() {
            return Some(("commit-message".into(), "Commit message is empty.".into()));
        }

        if trimmed.len() < 3 {
            return Some((
                "commit-message".into(),
                format!(
                    "Commit message too short ({} chars). Use a meaningful message.",
                    trimmed.len()
                ),
            ));
        }

        None
    }
}
