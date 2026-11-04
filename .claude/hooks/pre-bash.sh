#!/usr/bin/env bash
# PreToolUse(Bash): refuse destructive commands.
. "$(dirname "$0")/lib.sh"
cmd=$(json_str command)
[ -n "$cmd" ] || exit 0
# rm -rf outside build and temp folders
if printf '%s' "$cmd" | grep -Eq '(^|[;&|[:space:]])rm[[:space:]]+(-[a-zA-Z]*r[a-zA-Z]*f|-[a-zA-Z]*f[a-zA-Z]*r|-r[[:space:]]+-f|-f[[:space:]]+-r)'; then
  rest=$(printf '%s' "$cmd" | sed -E 's/.*rm[[:space:]]+-[a-zA-Z]+([[:space:]]+-[a-zA-Z]+)?//')
  for t in $rest; do
    case "$t" in
      target|target/*|./target|./target/*|/tmp/*|dist|dist/*|./dist|./dist/*|demo-data|./demo-data|data|./data|.claude/state/*|crates/server/assets/viewer*) ;;
      *) block "Refused: rm -rf on '$t'. Only build/temp folders (target, dist, demo-data, /tmp) may be removed this way." ;;
    esac
  done
fi
printf '%s' "$cmd" | grep -Eq 'git[[:space:]]+push.*(--force|[[:space:]]-f([[:space:]]|$)|--force-with-lease|[[:space:]]\+[^[:space:]]+)' && block "Refused: force-push is never allowed."
if printf '%s' "$cmd" | grep -Eq 'git[[:space:]]+reset[[:space:]]+--hard'; then
  [ "$(git -C "$ROOT" branch --show-current 2>/dev/null)" = "main" ] && block "Refused: git reset --hard on main."
fi
printf '%s' "$cmd" | grep -Eiq '(drop|truncate)[[:space:]]+(table|database)' && block "Refused: DROP/TRUNCATE."
printf '%s' "$cmd" | grep -Eq '(rm|mv|git[[:space:]]+rm)[^;&|]*\.github/(workflows|scripts)' && block "Refused: removing CI files. CI changes need the owner (create .claude/state/allow-ci and edit, don't delete)."
printf '%s' "$cmd" | grep -Eq -- '--no-verify|-c[[:space:]]+commit\.gpgsign=false' && block "Refused: don't skip hooks or signing."
exit 0
