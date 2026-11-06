#!/usr/bin/env bash
# Stop hook: (1) quick gate, (2) keep working through the claude-queue, up to 10 tasks a session.
# Allows stopping when: .claude/state/STOP exists; the last "continue" produced no new commit; 10 tasks are done;
# no open claude-queue issues; main is red and one fix attempt already happened; or anything errors (fail open).
. "$(dirname "$0")/lib.sh"
active=false
printf '%s' "$INPUT" | grep -Eq '"stop_hook_active"[[:space:]]*:[[:space:]]*true' && active=true
[ -f "$STATE/STOP" ] && { log "stop: STOP file"; exit 0; }

# (1) Quick gate: formatting. Block once so the agent fixes it; give up after one retry.
if command -v cargo >/dev/null 2>&1 && ! (cd "$ROOT" && cargo fmt --all --check >/dev/null 2>&1); then
  if [ ! -f "$STATE/gate-retry" ]; then
    touch "$STATE/gate-retry"
    block "cargo fmt --all --check fails. Run cargo fmt --all, then cargo xtask ci, and commit the fix before stopping."
  fi
  log "stop: fmt still failing after one retry"
  exit 0
fi
rm -f "$STATE/gate-retry"

# (2) Keep going.
head=$(git -C "$ROOT" rev-parse HEAD 2>/dev/null || echo none)
if $active && [ "$(cat "$STATE/last-continue-head" 2>/dev/null)" = "$head" ]; then
  log "stop: no new commit since the last continue"; exit 0
fi
n=$(cat "$STATE/tasks-done" 2>/dev/null || echo 0)
[ "${n:-0}" -ge 10 ] && { log "stop: $n tasks done"; exit 0; }
command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1 || { log "stop: gh unavailable"; exit 0; }
queued=$(cd "$ROOT" && gh issue list --label claude-queue --state open --json number -q 'length' 2>/dev/null) || { log "stop: gh issue list failed"; exit 0; }
[ "${queued:-0}" -gt 0 ] || { log "stop: queue empty"; exit 0; }
main=$(cd "$ROOT" && gh run list --branch main --workflow CI --limit 1 --json conclusion -q '.[0].conclusion' 2>/dev/null || echo unknown)
if [ "$main" = "failure" ]; then
  [ -f "$STATE/main-fix-tried" ] && { log "stop: main red after one fix attempt"; exit 0; }
  touch "$STATE/main-fix-tried"
  echo "$head" > "$STATE/last-continue-head"
  block "CI on main is red. Fixing main is the next task: find the failing check, fix it on a claude/ branch, open a PR."
fi
echo "$head" > "$STATE/last-continue-head"
block "Task finished. Update STATUS.md, then pick the next oldest open claude-queue issue and run the full loop (build, test, PR, gate green, screenshots if UI). Tasks done this session: ${n:-0} of 10."
