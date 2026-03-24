//! Git state helper module.
//!
//! Wraps `std::process::Command` calls to git for querying repository state.

use std::path::Path;
use std::process::Command;

/// Information about a git branch.
#[derive(Debug, Clone)]
pub struct BranchInfo {
    /// Branch name (e.g. "main", "feature/foo").
    pub name: String,
    /// Last commit date as ISO 8601 string, if available.
    pub last_commit_date: Option<String>,
    /// Whether the branch has a remote tracking branch.
    pub has_remote: bool,
}

/// Error type for git operations.
pub type GitError = Box<dyn std::error::Error>;
/// Result type for git operations.
pub type GitResult<T> = Result<T, GitError>;

/// Run a git command in the given directory and return stdout as a trimmed string.
fn run_git(dir: &Path, args: &[&str]) -> GitResult<String> {
    let output = Command::new("git").args(args).current_dir(dir).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(stderr.into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Returns the current branch name (e.g. "main").
/// Returns "HEAD" if in detached HEAD state.
pub fn current_branch(dir: &Path) -> GitResult<String> {
    run_git(dir, &["rev-parse", "--abbrev-ref", "HEAD"])
}

/// Returns true if the working tree has uncommitted changes (staged or unstaged).
pub fn is_dirty(dir: &Path) -> bool {
    run_git(dir, &["status", "--porcelain"])
        .map(|s| !s.is_empty())
        .unwrap_or(false)
}

/// Returns true if HEAD is detached (not on any branch).
pub fn is_detached_head(dir: &Path) -> bool {
    run_git(dir, &["symbolic-ref", "HEAD"]).is_err()
}

/// Returns the staged diff (what would be committed with `git commit`).
pub fn staged_diff(dir: &Path) -> GitResult<String> {
    run_git(dir, &["diff", "--cached"])
}

/// Returns the diff of working tree vs HEAD (approximates `git commit -a`).
pub fn working_diff(dir: &Path) -> GitResult<String> {
    run_git(dir, &["diff", "HEAD"])
}

/// Returns how many commits the current branch is behind its upstream.
/// Returns `None` if there is no upstream configured.
pub fn commits_behind_upstream(dir: &Path) -> GitResult<Option<usize>> {
    match run_git(dir, &["rev-list", "--count", "HEAD..@{u}"]) {
        Ok(count) => Ok(Some(count.parse::<usize>()?)),
        Err(_) => Ok(None),
    }
}

/// Lists all local branches with metadata.
pub fn list_branches(dir: &Path) -> GitResult<Vec<BranchInfo>> {
    // format: refname:short, committerdate:iso8601, upstream
    let output = run_git(
        dir,
        &[
            "for-each-ref",
            "--format=%(refname:short)\t%(committerdate:iso8601)\t%(upstream)",
            "refs/heads/",
        ],
    )?;

    let mut branches = Vec::new();
    for line in output.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        let name = parts.first().unwrap_or(&"").to_string();
        let date = parts
            .get(1)
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());
        let upstream = parts.get(2).map(|s| !s.is_empty()).unwrap_or(false);

        if !name.is_empty() {
            branches.push(BranchInfo {
                name,
                last_commit_date: date,
                has_remote: upstream,
            });
        }
    }

    Ok(branches)
}

/// Extracts the commit message from a git command string.
///
/// Parses `-m "..."` or `-m '...'` from the command. Returns `None` if no
/// `-m` flag is found.
pub fn extract_commit_message(command: &str) -> Option<String> {
    // Look for -m followed by a quoted string
    let bytes = command.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Find "-m"
        if i + 1 < len && bytes[i] == b'-' && bytes[i + 1] == b'm' {
            // Make sure -m is preceded by whitespace or start of string
            let preceded_ok = i == 0 || bytes[i - 1] == b' ';
            if !preceded_ok {
                i += 1;
                continue;
            }

            // Skip past "-m"
            i += 2;

            // Skip whitespace
            while i < len && bytes[i] == b' ' {
                i += 1;
            }

            if i >= len {
                return None;
            }

            // Check for quote character
            let quote = bytes[i];
            if quote == b'"' || quote == b'\'' {
                i += 1; // skip opening quote
                let start = i;
                while i < len && bytes[i] != quote {
                    if bytes[i] == b'\\' && i + 1 < len {
                        i += 1; // skip escaped char
                    }
                    i += 1;
                }
                let msg = &command[start..i];
                return Some(msg.to_string());
            }

            // No quote — take until next space or end
            let start = i;
            while i < len && bytes[i] != b' ' {
                i += 1;
            }
            return Some(command[start..i].to_string());
        }
        i += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_double_quoted() {
        assert_eq!(
            extract_commit_message(r#"git commit -m "hello world""#),
            Some("hello world".to_string())
        );
    }

    #[test]
    fn extract_single_quoted() {
        assert_eq!(
            extract_commit_message("git commit -m 'hello world'"),
            Some("hello world".to_string())
        );
    }

    #[test]
    fn extract_no_message() {
        assert_eq!(extract_commit_message("git commit --amend"), None);
    }

    #[test]
    fn extract_unquoted() {
        assert_eq!(
            extract_commit_message("git commit -m fix"),
            Some("fix".to_string())
        );
    }
}
