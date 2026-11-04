#!/usr/bin/env bash
# PostToolUse(Edit|Write): rustfmt the edited Rust file. Never fails.
. "$(dirname "$0")/lib.sh"
f=$(json_str file_path)
case "$f" in
  *.rs) command -v rustfmt >/dev/null 2>&1 && rustfmt --edition 2024 --config-path "$ROOT/rustfmt.toml" "$f" >/dev/null 2>&1 || true ;;
esac
exit 0
