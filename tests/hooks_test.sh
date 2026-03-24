#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TINSTAR="$SCRIPT_DIR/cli/target/debug/tinstar"
HOOKS="$SCRIPT_DIR/hooks"
PASS=0
FAIL=0

run_test() {
    local name="$1" expected_exit="$2" input="$3"
    actual_exit=0
    echo "$input" | TINSTAR="$TINSTAR" TINSTAR_PROJECT="$SCRIPT_DIR" "$HOOKS/pre-tool-use" >/dev/null 2>&1 || actual_exit=$?
    if [ "$actual_exit" = "$expected_exit" ]; then
        echo "  PASS: $name"
        PASS=$((PASS + 1))
    else
        echo "  FAIL: $name (expected exit $expected_exit, got $actual_exit)"
        FAIL=$((FAIL + 1))
    fi
}

echo "Building tinstar..."
(cd "$SCRIPT_DIR/cli" && cargo build --quiet)

echo ""
echo "=== pre-tool-use tests ==="
run_test "fast path: non-git command" 0 '{"tool_name":"Bash","tool_input":{"command":"ls -la"}}'
run_test "fast path: empty command" 0 '{"tool_name":"Bash","tool_input":{"command":""}}'
run_test "block: force push" 2 '{"tool_name":"Bash","tool_input":{"command":"git push --force origin main"}}'
run_test "block: no-verify" 2 '{"tool_name":"Bash","tool_input":{"command":"git commit --no-verify -m test"}}'
run_test "block: reset --hard" 2 '{"tool_name":"Bash","tool_input":{"command":"git reset --hard"}}'
run_test "allow: normal push" 0 '{"tool_name":"Bash","tool_input":{"command":"git push origin main"}}'
run_test "allow: git status" 0 '{"tool_name":"Bash","tool_input":{"command":"git status"}}'
run_test "allow: force-with-lease" 0 '{"tool_name":"Bash","tool_input":{"command":"git push --force-with-lease origin main"}}'

echo ""
echo "=== stop hook test ==="
STOP_EXIT=0
TINSTAR="$TINSTAR" TINSTAR_PROJECT="$SCRIPT_DIR" "$HOOKS/stop" >/dev/null 2>&1 || STOP_EXIT=$?
if [ "$STOP_EXIT" = "0" ] || [ "$STOP_EXIT" = "2" ]; then
    echo "  PASS: stop hook returns valid exit code ($STOP_EXIT)"
    PASS=$((PASS + 1))
else
    echo "  FAIL: stop hook returned unexpected exit $STOP_EXIT"
    FAIL=$((FAIL + 1))
fi

echo ""
echo "=== graceful degradation ==="
DEGRADE_EXIT=0
echo '{"tool_name":"Bash","tool_input":{"command":"git push --force"}}' | TINSTAR="/nonexistent/tinstar" TINSTAR_PROJECT="$SCRIPT_DIR" CLAUDE_PLUGIN_ROOT="$SCRIPT_DIR" "$HOOKS/run-hook.cmd" pre-tool-use >/dev/null 2>&1 || DEGRADE_EXIT=$?
if [ "$DEGRADE_EXIT" = "0" ]; then
    echo "  PASS: graceful degradation (no binary = exit 0)"
    PASS=$((PASS + 1))
else
    echo "  FAIL: degradation returned exit $DEGRADE_EXIT instead of 0"
    FAIL=$((FAIL + 1))
fi

echo ""
echo "Results: $PASS passed, $FAIL failed"
[ "$FAIL" = "0" ] || exit 1
