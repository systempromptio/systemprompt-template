#!/usr/bin/env bash
# Switches core between the committed git-rev pin (scripts/core-pin.sh) and a
# local core checkout. `on` appends a [patch."<core repo>"] table of path
# entries to every workspace manifest that names core, one entry per core
# crate that workspace's lockfile resolves, and hides the manifests and locks
# from git; `off` restores the committed pin. The workspaces are the root, the
# separate tests/, the phone apps' binding workspace (extensions/mobile/core),
# so the library a phone loads is built from the same checkout, and the
# desktop shell with its tests workspace (extensions/harness,
# extensions/harness/tests), so the shell is built from it too. The checkout
# must hold every crate a lockfile names: one without core's device crates
# cannot be switched on while the shell depends on them, and `on` names the
# missing crate before it changes anything.
#   scripts/core-patch.sh on <core checkout> | off | status
set -euo pipefail
cd "$(dirname "$0")/.."
url=https://github.com/systempromptio/systemprompt-core
workspaces=(. tests extensions/mobile/core extensions/harness extensions/harness/tests)
files=()
for w in "${workspaces[@]}"; do
    files+=("$w/Cargo.toml" "$w/Cargo.lock")
done
files=("${files[@]#./}")

status() {
    if grep -qxF "[patch.\"$url\"]" Cargo.toml; then echo path; else echo pin; fi
}

patch_table() {
    local manifest="$1" lock="$2" core="$3" rel
    # Why: `realpath --relative-to` is GNU-only; macOS's realpath has no such flag.
    rel="$(python3 -c 'import os, sys; print(os.path.relpath(sys.argv[1], os.path.dirname(os.path.abspath(sys.argv[2]))))' "$core" "$manifest")"
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
    core="$(cd "${2:?usage: scripts/core-patch.sh on <core checkout>}" && pwd -P)"
    [ -f "$core/systemprompt/Cargo.toml" ] || { echo "no systemprompt-core checkout at $core" >&2; exit 1; }
    [ "$(status)" = pin ] || { echo "a local core override is already on; run scripts/core-patch.sh off first" >&2; exit 1; }
    tables=()
    for w in "${workspaces[@]}"; do
        tables+=("$(patch_table "$w/Cargo.toml" "$w/Cargo.lock" "$core")")
    done
    git update-index --skip-worktree "${files[@]}"
    i=0
    for w in "${workspaces[@]}"; do
        printf '%s\n' "${tables[$i]}" >> "$w/Cargo.toml"
        i=$((i + 1))
    done
    echo "core override on: $core; the manifests and locks of ${workspaces[*]} are hidden from git"
    ;;
  off)
    git update-index --no-skip-worktree "${files[@]}"
    git checkout -- "${files[@]}"
    echo "core override off: the committed pin (bridge/CORE_REF, and the shell's own rev while it is apart) is back"
    ;;
  status) status ;;
  *) echo "usage: scripts/core-patch.sh on <core checkout> | off | status" >&2; exit 2 ;;
esac
