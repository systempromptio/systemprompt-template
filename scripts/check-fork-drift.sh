#!/usr/bin/env bash
# Report shared `.rs` files that differ between this repo and its sibling fork.
#
# The two trees share the great majority of their extension sources, but
# nothing enforced that they stay in step, so they drifted — and most of the
# drift was never a decision. It accumulated because each repo ran a source
# gate the other did not, so one side rewrote files the other never touched.
# Every such difference makes the next upstream merge noisier, and eventually
# makes it hard to tell an intentional fork change from an accident.
#
# This does not demand the trees be identical. It demands that every
# difference be *recorded*. `.fork-divergence` lists the files allowed to
# differ, each with a reason. A file that differs without an entry fails the
# check; an entry whose file no longer differs also fails, so the list can
# only shrink as files are re-converged.
#
# Point SIBLING_REPO at the other checkout. Without it the check is skipped,
# so CI — where only one repo is present — stays green.
set -uo pipefail

cd "$(dirname "$0")/.."

SIBLING_REPO="${SIBLING_REPO:-}"
ALLOWLIST="${ALLOWLIST:-.fork-divergence}"

if [ -z "$SIBLING_REPO" ]; then
    echo "check-fork-drift: SIBLING_REPO not set - skipping (single-repo run)"
    exit 0
fi
if [ ! -d "$SIBLING_REPO" ]; then
    echo "check-fork-drift: SIBLING_REPO '$SIBLING_REPO' is not a directory" >&2
    exit 2
fi

# Allowlist format: one path per line. `#` comments and blank lines ignored;
# a reason may follow the path after whitespace.
declare -A ALLOWED
if [ -f "$ALLOWLIST" ]; then
    while IFS= read -r line; do
        line="${line%%#*}"
        path="${line%%[[:space:]]*}"
        [ -n "$path" ] && ALLOWED["$path"]=1
    done < "$ALLOWLIST"
fi

# A file unique to one repo is scope, not drift, and is skipped below. The same
# fact shows up one level higher: the `mod.rs` that declares it exists in both
# trees and differs by exactly that declaration. Recording those in the
# allowlist would be recording the same decision twice, in a list that is
# supposed to shrink — so derive them instead.
#
# The test is deliberately narrow. Only `mod` declarations qualify, and only
# when the module really is absent from the other tree; a differing `use` or a
# re-export is a genuine difference and still needs an entry.
is_scope_only() {
    local rel="$1" dir line side name other
    dir="$(dirname "$rel")"
    while IFS= read -r line; do
        [[ "$line" =~ ^([<>])[[:space:]]*(pub(\([a-z]+\))?[[:space:]]+)?mod[[:space:]]+([a-z0-9_]+)\;[[:space:]]*$ ]] || return 1
        side="${BASH_REMATCH[1]}"
        name="${BASH_REMATCH[4]}"
        # `<` is a line only this tree has, so the module must be missing from
        # the sibling; `>` is the mirror image.
        if [ "$side" = "<" ]; then other="$SIBLING_REPO/$dir"; else other="$dir"; fi
        [ -f "$other/$name.rs" ] && return 1
        [ -f "$other/$name/mod.rs" ] && return 1
    done < <(diff "$rel" "$SIBLING_REPO/$rel" | grep '^[<>]')
    return 0
}

undocumented=()
divergent=()
scope=()

while IFS= read -r rel; do
    # Only files present in BOTH trees are shared; a file unique to one repo
    # is not drift, it is scope.
    [ -f "$SIBLING_REPO/$rel" ] || continue
    cmp -s "$rel" "$SIBLING_REPO/$rel" && continue
    if is_scope_only "$rel"; then
        scope+=("$rel")
        continue
    fi
    divergent+=("$rel")
    [ -n "${ALLOWED[$rel]:-}" ] || undocumented+=("$rel")
done < <(find extensions src -name '*.rs' -not -path '*/target/*' 2>/dev/null | sed 's|^\./||' | sort)

# An allowlist entry whose file now matches is stale: delete it, so the
# recorded debt reflects reality and shrinks as work lands.
stale=()
for path in "${!ALLOWED[@]}"; do
    [ -f "$path" ] || { stale+=("$path (no longer exists here)"); continue; }
    [ -f "$SIBLING_REPO/$path" ] || { stale+=("$path (no longer exists in sibling)"); continue; }
    cmp -s "$path" "$SIBLING_REPO/$path" && { stale+=("$path (now identical)"); continue; }
    is_scope_only "$path" && stale+=("$path (derived as scope, no entry needed)")
done

fail=0

if [ ${#undocumented[@]} -ne 0 ]; then
    echo "check-fork-drift: ${#undocumented[@]} shared file(s) differ without an entry in $ALLOWLIST:"
    printf '  %s\n' "${undocumented[@]}"
    echo
    echo "Either port the change to the other repo, or add the file to $ALLOWLIST"
    echo "with a one-line reason it is legitimately fork-specific."
    fail=1
fi

if [ ${#stale[@]} -ne 0 ]; then
    echo "check-fork-drift: ${#stale[@]} stale entr(ies) in $ALLOWLIST:"
    printf '  %s\n' "${stale[@]}"
    echo
    echo "These no longer differ. Remove them so the list keeps shrinking."
    fail=1
fi

[ "$fail" -ne 0 ] && exit 1

echo "check-fork-drift: OK - ${#divergent[@]} divergent file(s), all recorded in $ALLOWLIST" \
     "(plus ${#scope[@]} derived as scope)"
