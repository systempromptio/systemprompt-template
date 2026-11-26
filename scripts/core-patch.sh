#!/usr/bin/env bash
# Switches core between the committed git-rev pin (scripts/core-pin.sh) and a
# local core checkout. `on` appends a [patch."<core repo>"] table of path
# entries to both workspaces (the root and the separate tests/), one per core
# crate that workspace's lockfile resolves, and hides the manifests and locks
# from git; `off` restores the committed pin.
#   scripts/core-patch.sh on <core checkout> | off | status
set -euo pipefail
cd "$(dirname "$0")/.."
url=https://github.com/systempromptio/systemprompt-core
files=(Cargo.toml Cargo.lock tests/Cargo.toml tests/Cargo.lock)

status() {
    if grep -qxF "[patch.\"$url\"]" Cargo.toml; then echo path; else echo pin; fi
}

patch_table() {
    local manifest="$1" lock="$2" core="$3" rel
    rel="$(realpath --relative-to="$(dirname "$manifest")" "$core")"
    printf '\n[patch."%s"]\n' "$url"
    awk -v rel="$rel" -v url="$url" '
        FNR == NR {
            if (match($0, /^systemprompt[a-z-]* = \{ path = "[^"]*"/)) {
                split($0, kv, " = "); p = $0; sub(/^[^"]*"/, "", p); sub(/".*$/, "", p); dir[kv[1]] = p
            }
            next
        }
        /^\[\[package\]\]/ { name = ""; next }
        /^name = "/ { name = $3; gsub(/"/, "", name); next }
        /^source = "git\+/ && index($0, url) && name ~ /^systemprompt/ {
            d = (name == "systemprompt") ? "systemprompt" : dir[name]
            if (d == "") { print "core checkout has no crate " name > "/dev/stderr"; exit 1 }
            print name " = { path = \"" rel "/" d "\" }"
        }
    ' "$core/Cargo.toml" "$lock" | sort -u
}

case "${1:-status}" in
  on)
    core="$(realpath "${2:?usage: scripts/core-patch.sh on <core checkout>}")"
    [ -f "$core/systemprompt/Cargo.toml" ] || { echo "no systemprompt-core checkout at $core" >&2; exit 1; }
    [ "$(status)" = pin ] || { echo "a local core override is already on; run scripts/core-patch.sh off first" >&2; exit 1; }
    root_table="$(patch_table Cargo.toml Cargo.lock "$core")"
    tests_table="$(patch_table tests/Cargo.toml tests/Cargo.lock "$core")"
    git update-index --skip-worktree "${files[@]}"
    printf '%s\n' "$root_table" >> Cargo.toml
    printf '%s\n' "$tests_table" >> tests/Cargo.toml
    echo "core override on: $core; the root and tests/ manifests and locks are hidden from git"
    ;;
  off)
    git update-index --no-skip-worktree "${files[@]}"
    git checkout -- "${files[@]}"
    echo "core override off: the committed pin (bridge/CORE_REF) is back"
    ;;
  status) status ;;
  *) echo "usage: scripts/core-patch.sh on <core checkout> | off | status" >&2; exit 2 ;;
esac
