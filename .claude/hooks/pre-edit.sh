#!/usr/bin/env bash
# PreToolUse(Edit|Write): refuse edits to secrets, licence files, the lock file and CI (unless allowed).
. "$(dirname "$0")/lib.sh"
f=$(json_str file_path)
[ -n "$f" ] || f=$(json_str notebook_path)
[ -n "$f" ] || exit 0
f=$(printf '%s' "$f" | tr '\\' '/' | sed 's#//*#/#g')
base=${f##*/}
case "$base" in
  .env|.env.*|*.pem|*.key|*.p12|*.pfx|id_rsa*|id_ed25519*) block "Refused: $base may hold secrets and never belongs in this public repository." ;;
  LICENSE-MIT|LICENSE-APACHE|LICENSE*) block "Refused: licence files are the owner's decision. Flag the question instead." ;;
  Cargo.lock) [ -f "$STATE/allow-lockfile" ] || block "Refused: Cargo.lock changes only in a dependency task (say so, then touch .claude/state/allow-lockfile)." ;;
esac
case "$f" in
  */.github/workflows/*|*/.github/scripts/*|.github/workflows/*|.github/scripts/*)
    [ -f "$STATE/allow-ci" ] || block "Refused: CI and public-guard changes need the owner (touch .claude/state/allow-ci only when the task is a CI change)." ;;
esac
exit 0
