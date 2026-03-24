use super::Rule;

pub struct NoVerifyRule;

impl Rule for NoVerifyRule {
    fn name(&self) -> &str {
        "no-verify"
    }

    fn check_command(&self, command: &str) -> Option<(String, String)> {
        let args: Vec<&str> = command.split_whitespace().collect();

        // Must be a git command
        if args.is_empty() || args[0] != "git" {
            return None;
        }

        if args.iter().any(|a| *a == "--no-verify") {
            return Some((
                "no-verify".into(),
                "--no-verify skips git hooks. Hooks exist to enforce project standards.".into(),
            ));
        }

        None
    }
}
