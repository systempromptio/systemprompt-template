#!/usr/bin/env bash
# bridge/CORE_REF names the systemprompt-core ref CI checks out beside this
# repo: bridge/ depends on systemprompt-bridge by path, and the active
# [patch.crates-io] on `next` resolves through it. Two rules keep it honest:
#
#   * the file exists and holds exactly one ref (tag or 40-char SHA);
#   * when [patch.crates-io] is COMMENTED (the `main` steady state), the ref
#     must be the tag of the published core the pins name, `v<pin>` — so the
#     bridge is proven against the same core the server crates come from.
#
# While the patch is active, also verify the sibling checkout and every
# workspace lockfile resolve to the pinned core version.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FILE="$REPO_ROOT/bridge/CORE_REF"

[ -f "$FILE" ] || { echo "check-core-ref: $FILE is missing" >&2; exit 1; }
ref="$(tr -d '[:space:]' < "$FILE")"
[ -n "$ref" ] || { echo "check-core-ref: $FILE is empty" >&2; exit 1; }
[ "$(wc -l < "$FILE")" -le 1 ] || { echo "check-core-ref: $FILE must hold a single ref" >&2; exit 1; }

case "$ref" in
    v[0-9]*.[0-9]*.[0-9]*) ;;
    *)
        [[ "$ref" =~ ^[0-9a-f]{40}$ ]] || {
            echo "check-core-ref: '$ref' is neither a vX.Y.Z tag nor a 40-char SHA" >&2; exit 1; }
        ;;
esac

if ! grep -qE '^\[patch\.crates-io\]' "$REPO_ROOT/Cargo.toml"; then
    pin="$(sed -n 's/^systemprompt = { version = "\([0-9.]*\)".*/\1/p' "$REPO_ROOT/Cargo.toml" | head -1)"
    [ -n "$pin" ] || { echo "check-core-ref: could not read the systemprompt pin from Cargo.toml" >&2; exit 1; }
    [ "$ref" = "v$pin" ] || {
        echo "check-core-ref: [patch.crates-io] is commented (published-core mode) but" >&2
        echo "  bridge/CORE_REF = $ref while Cargo.toml pins systemprompt $pin." >&2
        echo "  Set bridge/CORE_REF to v$pin so the bridge builds against the released core." >&2
        exit 1
    }
fi
if grep -qE '^\[patch\.crates-io\]' "$REPO_ROOT/Cargo.toml"; then
    bash "$REPO_ROOT/scripts/check-core-resolution.sh"
fi
echo "check-core-ref: $ref ok"
