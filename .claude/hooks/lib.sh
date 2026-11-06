# Shared by the hooks. Every hook fails open: an unexpected error allows the action and is logged.
ROOT="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
STATE="$ROOT/.claude/state"
mkdir -p "$STATE" 2>/dev/null || true
log() { printf '%s %s\n' "$(date -u +%FT%TZ)" "$*" >> "$STATE/hook.log" 2>/dev/null || true; }
# The JSON on stdin, flattened to one line.
INPUT=$(cat 2>/dev/null | tr '\n' ' ' || true)
# json_str KEY: the first string value for KEY, JSON escapes left as they are (good enough for hook input; no jq needed).
json_str() {
  local re='s/.*"KEY"[[:space:]]*:[[:space:]]*"\(\([^"\\]\|\\.\)*\)".*/\1/p'
  printf '%s' "$INPUT" | sed -n "${re/KEY/$1}" | head -n 1
}
block() { echo "$*" >&2; log "blocked: $*"; exit 2; }
trap 'log "hook error in $0 (allowed)"; exit 0' ERR
