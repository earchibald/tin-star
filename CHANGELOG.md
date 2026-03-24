# Changelog

## 0.1.4

- Add: 10 new secret detection patterns — GitHub PAT, GitLab PAT, Anthropic, OpenAI, Google API, Slack, Stripe, JWT, database URLs, npm tokens
- Add: `tinstar:ignore` inline annotation to suppress false positives on a line (same convention as gitleaks, trufflehog)
- Add: 13 pattern-specific tests in e2e suite (35 E2E + 10 hook = 45 total)

## 0.1.3

- Fix: graceful degradation test properly hides binary
- All 32 tests passing (22 E2E + 10 hook), 6 live hook blocks verified

## 0.1.2

- Fix: `--dirty` boolean flag in post-tool-use hook (was passing string, now conditional)
- Add: versioned E2E test suite (`tests/e2e_test.sh`, 22 tests)
- Fix: performance test measures shell fast-path, not binary startup

## 0.1.1

- Fix: handle chained commands in hook fast-path (`cd /path && git ...` now detected)
- Fix: resolve all clippy warnings, apply cargo fmt

## 0.1.0

- Initial release: 8 rules, 10 CLI commands, 4 hook scripts, sweep skill
