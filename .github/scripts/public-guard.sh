#!/usr/bin/env bash
# public-guard: this repository is public. Fails if the tracked files or the commits contain anything that must
# stay private: paths or crate names of private add-ons, the names of private products, private keys, .env or key
# files, tokens, or a commit author other than the owner.
#
#   .github/scripts/public-guard.sh [ROOT]          check ROOT (default: this repository)
#   .github/scripts/public-guard.sh --self-test     prove the guard fails on planted files and passes on a clean repo
#
# Commits checked: GUARD_RANGE (for example "BASE..HEAD" on a pull request), or every commit reachable from HEAD.
# The forbidden words are put together from pieces so this file does not trip its own check.
set -euo pipefail

# Owner identities allowed as commit authors (email, lowercase).
ALLOWED_AUTHORS=" bwattendorf@gmail.com 1148265+bwattendorf@users.noreply.github.com bwattendorf@users.noreply.github.com "

# git grep that reports its own failure as a hit (exit 1 = no match is fine; anything above is an error).
gg() {
  local out rc=0
  out=$(git grep "$@") || rc=$?
  if [ "$rc" -gt 1 ]; then echo "git grep failed (exit $rc): $*"; else printf '%s' "$out"; fi
}

guard() {
  local root=$1 fail=0 hits
  cd "$root"
  err() { echo "::error::public-guard: $*"; fail=1; }

  local ent="enter""prise" bk="badge[ _-]?""kiosk" drone="dro""ne" tadmin="tenant[ _-]?""admin" sic="sicu""net"
  local begin="-----BE""GIN"

  # 1. Paths and crate names of private add-ons. docs/core-vs-enterprise.md is the one public file about the split.
  hits=$(git ls-files | grep -i -E "$ent|$bk|$drone|$tadmin" | grep -v -x -F "docs/core-vs-${ent}.md" || true)
  [ -z "$hits" ] || err "private add-on paths: $(echo "$hits" | tr '\n' ' ')"
  hits=$(git ls-files -z -- '*Cargo.toml' | xargs -0 -r grep -n -i -E "^name *= *\".*($ent|$bk|$drone|$tadmin)" || true)
  [ -z "$hits" ] || err "private add-on crate names: $hits"

  # 2. Names of private products, anywhere in tracked text.
  hits=$(gg -n -I -i -E -e "$bk|$sic" -- .)
  [ -z "$hits" ] || err "private product names:"$'\n'"$hits"

  # 3. Private keys, .env and key files, tokens. AKIAIOSFODNN7EXAMPLE is the public AWS documentation sample.
  hits=$(gg -n -I -E -e "$begin ([A-Z0-9]+ )*PRIVATE KEY" -- .)
  [ -z "$hits" ] || err "private key material:"$'\n'"$hits"
  hits=$(git ls-files | grep -i -E '(^|/)\.env($|\.)|\.(pem|key|p12|pfx|jks|keystore)$|(^|/)id_(rsa|dsa|ecdsa|ed25519)$' || true)
  [ -z "$hits" ] || err ".env or key files: $(echo "$hits" | tr '\n' ' ')"
  hits=$(gg -n -I -E -e 'gh[pousr]_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{22,}|AKIA[0-9A-Z]{16}|(sk|rk)_live_[0-9A-Za-z]{10,}|xox[abprs]-[0-9A-Za-z-]{10,}' -- . \
    | grep -v 'AKIAIOSFODNN7EXAMPLE' || true)
  [ -z "$hits" ] || err "tokens or access keys:"$'\n'"$hits"

  # 4. Commit authors.
  local range=${GUARD_RANGE:-HEAD} line email
  while IFS= read -r line; do
    email=$(printf '%s' "${line##* <}" | tr -d '>' | tr '[:upper:]' '[:lower:]')
    case "$ALLOWED_AUTHORS" in
      *" $email "*) ;;
      *) err "commit by someone other than the owner: $line" ;;
    esac
  done < <(git log --no-merges --format='%h %an <%ae>' "$range")

  if [ "$fail" -eq 0 ]; then echo "public-guard: clean"; fi
  return "$fail"
}

self_test() {
  local tmp ok=0
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' RETURN
  mk() { # mk DIR AUTHOR_EMAIL FILE CONTENT
    git init -q "$1"
    printf '%s\n' "$4" > "$1/$3"
    git -C "$1" add -A
    git -C "$1" -c user.name=t -c user.email="$2" commit -q -m t
  }
  mk "$tmp/clean" bwattendorf@gmail.com README.md "construction management"
  mk "$tmp/name" bwattendorf@gmail.com README.md "works with Badge""Kiosk"
  mk "$tmp/vendor" bwattendorf@gmail.com notes.md "Sicu""net connector"
  mk "$tmp/key" bwattendorf@gmail.com k.txt "-----BE""GIN RSA PRIVATE KEY-----"
  mk "$tmp/env" bwattendorf@gmail.com .env "X=1"
  mk "$tmp/token" bwattendorf@gmail.com cfg.toml "token = \"gh""p_$(printf 'a%.0s' {1..36})\""
  mk "$tmp/path" bwattendorf@gmail.com "enter""prise.rs" "fn main() {}"
  mk "$tmp/author" someone@example.com README.md "construction management"
  if (GUARD_RANGE=HEAD guard "$tmp/clean"); then echo "self-test: clean repo passes"; else echo "self-test: clean repo FAILED"; ok=1; fi
  for case in name vendor key env token path author; do
    if (GUARD_RANGE=HEAD guard "$tmp/$case" >/dev/null 2>&1); then
      echo "self-test: '$case' was NOT caught"; ok=1
    else
      echo "self-test: '$case' caught"
    fi
  done
  return "$ok"
}

if [ "${1:-}" = "--self-test" ]; then
  self_test
else
  guard "${1:-$(git rev-parse --show-toplevel)}"
fi
