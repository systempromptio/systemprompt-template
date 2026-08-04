#!/usr/bin/env bash
# Gate: every [workspace.dependencies] entry must be inherited by at least one
# workspace member (or the root package) via `workspace = true`.
# cargo-machete audits each package's [dependencies]; it never looks at the
# workspace table, so a dep that loses its last consumer stays declared forever.
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0
in_table=0
while IFS= read -r line; do
    case "$line" in
        "[workspace.dependencies]") in_table=1; continue ;;
        "["*) in_table=0 ;;
    esac
    [ "$in_table" -eq 1 ] || continue
    dep=$(printf '%s' "$line" | sed -n 's/^\([A-Za-z0-9_-]\+\)[[:space:]]*=.*/\1/p')
    [ -n "$dep" ] || continue
    if ! grep -rqE "^${dep}([[:space:]]*=.*workspace[[:space:]]*=[[:space:]]*true|\.workspace[[:space:]]*=[[:space:]]*true)" \
        --include=Cargo.toml Cargo.toml extensions src 2>/dev/null; then
        echo "unused workspace dependency: $dep (no member inherits it)"
        fail=1
    fi
done < Cargo.toml

if [ "$fail" -ne 0 ]; then
    echo "workspace dependencies FAILED: prune the entries above from [workspace.dependencies]"
    exit 1
fi
echo "workspace dependencies OK"
