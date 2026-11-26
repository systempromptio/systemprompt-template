#!/usr/bin/env bash
# bridge/CORE_REF names the systemprompt-core the build resolves and the one CI
# checks out beside this repo for core's contract scanners.
#
#   * the file exists and holds exactly one ref (tag or 40-char SHA);
#   * with core named as a git dependency (core pinned by git rev), the ref is
#     that rev and every manifest entry and lockfile names it
#     (scripts/core-pin.sh --check);
#   * otherwise (the published-core steady state on `main`), the ref is the
#     tag of the published core the pins name, `v<pin>`.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FILE="$REPO_ROOT/bridge/CORE_REF"

[ -f "$FILE" ] || { echo "check-core-ref: $FILE is missing" >&2; exit 1; }
ref="$(tr -d '[:space:]' < "$FILE")"
[ -n "$ref" ] || { echo "check-core-ref: $FILE is empty" >&2; exit 1; }
[ "$(wc -l < "$FILE")" -le 1 ] || { echo "check-core-ref: $FILE must hold a single ref" >&2; exit 1; }

if grep -qF 'git = "https://github.com/systempromptio/systemprompt-core"' "$REPO_ROOT/Cargo.toml"; then
    if [ "$(bash "$REPO_ROOT/scripts/core-patch.sh" status)" = path ]; then
        echo "check-core-ref: a local core override is on (scripts/core-patch.sh); the committed pin is $ref"
        exit 0
    fi
    bash "$REPO_ROOT/scripts/core-pin.sh" --check
else
    [[ "$ref" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
        echo "check-core-ref: '$ref' is not a vX.Y.Z tag and Cargo.toml pins no core git rev" >&2; exit 1; }
    pin="$(sed -n 's/^systemprompt = { version = "\([0-9.]*\)".*/\1/p' "$REPO_ROOT/Cargo.toml" | head -1)"
    [ -n "$pin" ] || { echo "check-core-ref: could not read the systemprompt pin from Cargo.toml" >&2; exit 1; }
    [ "$ref" = "v$pin" ] || {
        echo "check-core-ref: bridge/CORE_REF = $ref while Cargo.toml pins published systemprompt $pin;" >&2
        echo "  set it to v$pin so CI checks out the released core." >&2
        exit 1
    }
fi
echo "check-core-ref: $ref ok"
