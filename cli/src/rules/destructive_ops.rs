use super::Rule;

pub struct DestructiveOpsRule;

impl Rule for DestructiveOpsRule {
    fn name(&self) -> &str {
        "destructive-ops"
    }

    fn check_command(&self, command: &str) -> Option<(String, String)> {
        let args: Vec<&str> = command.split_whitespace().collect();

        if args.len() < 2 || args[0] != "git" {
            return None;
        }

        let subcommand = args[1];

        match subcommand {
            // git reset --hard
            "reset" => {
                if args.contains(&"--hard") {
                    return Some((
                        "destructive-ops".into(),
                        "git reset --hard discards uncommitted changes.".into(),
                    ));
                }
            }
            // git checkout .
            "checkout" => {
                if args.contains(&".") {
                    return Some((
                        "destructive-ops".into(),
                        "git checkout . discards all unstaged changes.".into(),
                    ));
                }
            }
            // git clean -f
            "clean" => {
                if args.iter().any(|a| a.contains('f')) {
                    return Some((
                        "destructive-ops".into(),
                        "git clean -f permanently deletes untracked files.".into(),
                    ));
                }
            }
            // git restore .
            "restore" => {
                if args.contains(&".") {
                    return Some((
                        "destructive-ops".into(),
                        "git restore . discards all unstaged changes.".into(),
                    ));
                }
            }
            _ => {}
        }

        None
    }
}
