use super::Rule;

pub struct ForcePushRule;

impl Rule for ForcePushRule {
    fn name(&self) -> &str {
        "force-push"
    }

    fn check_command(&self, command: &str) -> Option<(String, String)> {
        let args: Vec<&str> = command.split_whitespace().collect();

        // Must be a git push command
        if args.len() < 2 || args[0] != "git" || args[1] != "push" {
            return None;
        }

        // --force-with-lease is explicitly allowed — check first
        if args.iter().any(|a| a.starts_with("--force-with-lease")) {
            return None;
        }

        // Check for --force or -f
        if args.iter().any(|a| *a == "--force" || *a == "-f") {
            return Some((
                "force-push".into(),
                "Force push detected. Use --force-with-lease instead.".into(),
            ));
        }

        None
    }
}
