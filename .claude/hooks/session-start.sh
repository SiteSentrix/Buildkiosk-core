#!/usr/bin/env bash
# SessionStart: reset the task counter on a new session; after a compaction, repeat the key rules.
. "$(dirname "$0")/lib.sh"
case "${1:-}" in
  startup) echo 0 > "$STATE/tasks-done"; rm -f "$STATE/last-continue-head" "$STATE/main-fix-tried" "$STATE/gate-retry" ;;
  compact) cat <<'TXT'
Key rules (AGENTS.md):
- PUBLIC repo, single-company core: no private add-on code, secrets, customer data or .env files; public-guard CI enforces it.
- One task = one issue = one branch claude/issue-N = one PR. Gate before every commit: cargo xtask ci.
- No panics in shipped code (no unwrap/expect/panic!/indexing outside tests). Everything is a command, permissions in the engine.
- No JavaScript; the journal is append-only; schema changes are additive.
- Clean-room: never use or copy any vendor's software, text or data; name no vendor anywhere.
- Commits authored by Bruce Wattendorf <bwattendorf@gmail.com>, ending with Co-Authored-By: Claude <noreply@anthropic.com>.
- Update STATUS.md, ROADMAP.md and docs/gaps.md in the same PR. Don't merge your own PR.
- To stop the keep-going loop early: create .claude/state/STOP.
TXT
  ;;
esac
exit 0
