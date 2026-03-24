#!/usr/bin/env bash
set -euo pipefail

HOOK_NAME="$1"
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"

TINSTAR="${HOME}/.tinstar/bin/tinstar"
if [ ! -x "$TINSTAR" ]; then
    TINSTAR="$(command -v tinstar 2>/dev/null || true)"
fi

export TINSTAR
export TINSTAR_ROOT="${CLAUDE_PLUGIN_ROOT:-$HOOK_DIR/..}"
export TINSTAR_PROJECT="${PWD}"

if [ "${TINSTAR_DEBUG:-0}" = "1" ]; then
    mkdir -p "${HOME}/.tinstar"
    STDIN_CONTENT="$(cat)"
    echo "[$(date -Iseconds)] hook=$HOOK_NAME stdin_bytes=${#STDIN_CONTENT}" >> "${HOME}/.tinstar/debug.log"
    echo "$STDIN_CONTENT" | "$HOOK_DIR/$HOOK_NAME"
    EXIT_CODE=$?
    echo "[$(date -Iseconds)] hook=$HOOK_NAME exit=$EXIT_CODE" >> "${HOME}/.tinstar/debug.log"
    exit $EXIT_CODE
fi

if [ "$HOOK_NAME" = "session-start" ]; then
    exec "$HOOK_DIR/$HOOK_NAME"
fi

if [ -z "${TINSTAR:-}" ] || [ ! -x "${TINSTAR:-}" ]; then
    exit 0
fi

exec "$HOOK_DIR/$HOOK_NAME"
