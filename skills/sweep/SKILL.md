---
name: sweep
description: Run branch hygiene scan — stale branches, divergence, naming violations. Use /loop 30m /sweep for background monitoring.
---

Run `tinstar sweep` in the current project directory and report the results.

If issues are found, summarize them and suggest actions:
- Stale branches: suggest deletion with `git branch -d <name>`
- Orphan tracking branches: suggest `git remote prune origin`
- Naming violations: note which branches don't match the configured pattern
- Divergence: suggest `git pull --rebase` to sync with upstream

If no issues are found, report that branch hygiene is clean.

For background monitoring during long sessions, users can run:
```
/loop 30m /sweep
```
