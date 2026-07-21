#!/usr/bin/env bash
# Report `pub` repository functions with no call site.
#
# rustc and clippy only see one crate at a time, so a `pub fn` that nothing
# calls still compiles cleanly — dead query code accumulates silently. This
# walks every `pub fn` under the repository layer and looks for a real call.
#
# Two things make a naive grep wrong, and both are handled here:
#
#   1. Aliased re-exports. `pub use routes::update_route as update_gateway_route`
#      means every call site says `update_gateway_route(...)`, so searching for
#      the defining name finds nothing and the function looks dead. Aliases are
#      collected first and searched alongside the original name.
#
#   2. Sibling forks. This repo and its downstream fork share ~96% of these
#      files but not their callers: a function dead here can be live there.
#      Point SIBLING_REPO at the fork and a symbol is only reported when it is
#      dead in BOTH trees. Without it the check still runs, but single-repo.
#
# Re-export lines and intra-doc links (`[`fetch_x`]`) are not call sites and do
# not count.
#
# Exemption: annotate a deliberately-unused public entry point with
# `// lint-ok: unused-pub` on the line above it, and say why.
set -uo pipefail

REPO_DIR="${REPO_DIR:-extensions/web/admin/src/repositories}"
SIBLING_REPO="${SIBLING_REPO:-}"

[ -d "$REPO_DIR" ] || { echo "check-dead-repository-code: no $REPO_DIR - nothing to check"; exit 0; }

roots_for() {
    local base="$1"
    for d in extensions src tests bridge/src; do
        [ -d "$base/$d" ] && printf '%s\n' "$base/$d"
    done
}

# name -> also-search-as (aliases introduced by `pub use ... as ...`)
declare -A ALIAS
while IFS=' ' read -r orig alias; do
    [ -n "${orig:-}" ] && [ -n "${alias:-}" ] && ALIAS["$orig"]+=" $alias"
done < <(grep -rhoE '\b[A-Za-z0-9_]+ as [A-Za-z0-9_]+' "$REPO_DIR" --include='*.rs' 2>/dev/null \
         | sed -E 's/ as / /' | sort -u)

# Functions annotated `// lint-ok: unused-pub` on the preceding line are
# deliberate entry points and are never reported.
declare -A EXEMPT
while IFS= read -r fn; do
    [ -n "$fn" ] && EXEMPT["$fn"]=1
done < <(find "$REPO_DIR" -name '*.rs' -exec awk '
    /lint-ok: unused-pub/ { skip = 1; next }
    skip && match($0, /^[[:space:]]*pub (async )?fn [a-z_0-9]+/) {
        sub(/.*fn /, ""); sub(/[^a-z_0-9].*/, ""); print; skip = 0; next
    }
    { skip = 0 }
' {} + 2>/dev/null | sort -u)

DEAD=()
while IFS= read -r fn; do
    [ -z "$fn" ] && continue
    [ -n "${EXEMPT[$fn]:-}" ] && continue
    names="$fn${ALIAS[$fn]:-}"
    hits=0
    for base in . $SIBLING_REPO; do
        [ -d "$base" ] || continue
        while IFS= read -r root; do
            for n in $names; do
                c=$(grep -rnE "\b${n}\s*\(" "$root" --include='*.rs' 2>/dev/null \
                    | grep -vE 'pub (async )?fn |^\s*//' | grep -vc '^$' || true)
                hits=$((hits + c))
            done
        done < <(roots_for "$base")
    done
    [ "$hits" -eq 0 ] && DEAD+=("$fn")
done < <(grep -rhoE '^\s*pub (async )?fn [a-z_0-9]+' "$REPO_DIR" --include='*.rs' 2>/dev/null \
         | sed 's/.*fn //' | sort -u)

if [ "${#DEAD[@]}" -gt 0 ]; then
    echo "check-dead-repository-code: ${#DEAD[@]} public function(s) with no call site."
    if [ -z "$SIBLING_REPO" ]; then
        echo "NOTE: SIBLING_REPO is unset, so downstream forks were not consulted."
    fi
    echo "Delete them, or annotate with '// lint-ok: unused-pub' and a reason:"
    printf '    %s\n' "${DEAD[@]}"
    exit 1
fi

echo "check-dead-repository-code: OK (every public repository function is called)"
