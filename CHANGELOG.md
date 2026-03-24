# Changelog

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
