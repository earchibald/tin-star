use super::Rule;

pub struct CommitToMain;

impl CommitToMain {
    /// Check if a git commit command is targeting a protected branch (main/master).
    /// Returns `Some((rule_name, reason))` if the rule fires.
    pub fn check_with_branch(&self, command: &str, branch: &str) -> Option<(String, String)> {
        let args: Vec<&str> = command.split_whitespace().collect();

        // Must be a git commit command
        if args.len() < 2 || args[0] != "git" || args[1] != "commit" {
            return None;
        }

        if branch == "main" || branch == "master" {
            Some((
                "commit-to-main".into(),
                format!(
                    "Committing directly to '{}'. Use a feature branch instead.",
                    branch
                ),
            ))
        } else {
            None
        }
    }
}

impl Rule for CommitToMain {
    fn name(&self) -> &str {
        "commit-to-main"
    }

    /// Command-only check always returns None — this rule needs branch context.
    fn check_command(&self, _command: &str) -> Option<(String, String)> {
        None
    }
}
