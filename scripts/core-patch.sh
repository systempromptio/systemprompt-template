#!/usr/bin/env bash
# Switches the working tree between the committed manifests (core override set
# inert) and a local build against the sibling core checkout they name. Covers
# every workspace that carries the override table: the root and the desktop
# shell (extensions/harness), each with its own manifest and lockfile.
#   scripts/core-patch.sh on|off|status
# status prints live or inert. When the workspaces disagree it prints mixed,
# names each manifest on stderr and exits 1; on or off puts them back in step.
set -euo pipefail
cd "$(dirname "$0")/.."
workspaces=(. extensions/harness)
inert='[workspace.metadata.unreleased-core-patch]'
live='[patch.crates-io]'
inert_re='\[workspace\.metadata\.unreleased-core-patch\]'
live_re='\[patch\.crates-io\]'

manifests=() files=()
for workspace in "${workspaces[@]}"; do
    manifest="$workspace/Cargo.toml"
    manifest="${manifest#./}"
    [ -f "$manifest" ] || { echo "core-patch: $manifest does not exist" >&2; exit 1; }
    manifests+=("$manifest")
    files+=("$manifest" "${manifest%.toml}.lock")
done

state() {
    if grep -qxF "$live" "$1"; then echo live
    elif grep -qxF "$inert" "$1"; then echo inert
    else echo "core-patch: $1 has neither $inert nor $live" >&2; exit 1
    fi
}
# Why: GNU sed takes -i with no argument and BSD sed requires one; a suffix
# attached to the flag is the one spelling both accept.
rename() { sed -i.bak "s|^$2\$|$3|" "$1" && rm -f "$1.bak"; }
# Why: git can only hide a file it tracks, and a lockfile cargo has not been
# asked to write yet is not one.
tracked() { git ls-files --error-unmatch -- "$1" > /dev/null 2>&1; }
changed=()
skip_worktree() {
    local file
    for file in "${files[@]}"; do
        if tracked "$file"; then git update-index "$1" "$file"; changed+=("$file"); fi
    done
}

states=()
for manifest in "${manifests[@]}"; do states+=("$(state "$manifest")"); done

case "${1:-status}" in
  on)
    for index in "${!manifests[@]}"; do
        if [ "${states[$index]}" = inert ]; then rename "${manifests[$index]}" "$inert_re" "$live"; fi
    done
    skip_worktree --skip-worktree
    echo "core patch live; hidden from git: ${changed[*]:-nothing tracked}"
    ;;
  off)
    skip_worktree --no-skip-worktree
    for index in "${!manifests[@]}"; do
        if [ "${states[$index]}" = live ]; then rename "${manifests[$index]}" "$live_re" "$inert"; fi
    done
    echo "core patch inert; visible to git: ${changed[*]:-nothing tracked}"
    ;;
  status)
    for index in "${!manifests[@]}"; do
        [ "${states[$index]}" = "${states[0]}" ] && continue
        echo mixed
        for other in "${!manifests[@]}"; do echo "core-patch: ${manifests[$other]} is ${states[$other]}" >&2; done
        exit 1
    done
    echo "${states[0]}"
    ;;
  *) echo "usage: scripts/core-patch.sh on|off|status" >&2; exit 2 ;;
esac
