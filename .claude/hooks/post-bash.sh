#!/usr/bin/env bash
# PostToolUse(Bash): count tasks (a PR opened or merged) for the keep-going Stop hook.
. "$(dirname "$0")/lib.sh"
cmd=$(json_str command)
if printf '%s' "$cmd" | grep -Eq 'gh[[:space:]]+pr[[:space:]]+(create|merge)'; then
  n=$(cat "$STATE/tasks-done" 2>/dev/null || echo 0)
  echo $(( ${n:-0} + 1 )) > "$STATE/tasks-done"
fi
exit 0
