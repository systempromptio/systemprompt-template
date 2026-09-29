#!/usr/bin/env bash
# Sync every systemprompt-core crate pin to one published core version, and
# point bridge/CORE_REF at its tag.
#
# This repo versions in lockstep with core: the workspace version, the helm
# chart's appVersion and the deploy catalogue image pins all name the core
# release they ship. Those product pins belong to
# scripts/sync-release-version.sh; `just core-bump X.Y.Z` runs both scripts.
# This one owns the crate pins and bridge/CORE_REF.
#
#   scripts/sync-core-version.sh 0.61.0          # apply
#   scripts/sync-core-version.sh 0.61.0 --check  # verify only (CI guard)
#   scripts/sync-core-version.sh --check         # verify against the root pin
#
# Covered pins:
#   Cargo.toml                 systemprompt/-security/-users/-content/
#                              -marketplace/-extension/-api/-evaluation
#   tests/Cargo.toml           the same set (separate workspace)
#   any other systemprompt* pin in a Cargo.toml (residual sweep)
#   bridge/CORE_REF            v<version>, when [patch.crates-io] is inactive
#
# macOS + Linux compatible (no GNU-only sed flags).
set -eu

if [ "${1:-}" = "--check" ]; then
    VERSION="$(sed -n 's/^systemprompt = { version = "\([0-9.]*\)".*/\1/p' "$(dirname "$0")/../Cargo.toml" | head -1)"
    MODE=--check
else
    VERSION="${1:?usage: sync-core-version.sh <core-version> [--check] | --check}"
    MODE="${2:-apply}"
fi
cd "$(dirname "$0")/.."

case "$VERSION" in
  *[!0-9.]*|*..*|.*|*.) echo "ERROR: '$VERSION' is not a plain semver (X.Y.Z)"; exit 1 ;;
esac
IFS=. read -r MAJ MIN PATCH <<EOV
$VERSION
EOV
: "${PATCH:?ERROR: version must have three components}"

fail=0

check_or_apply() { # $1=file $2=sed-expr $3=expect-regex $4=label
    local file="$1" sedexpr="$2" expect="$3" label="$4"
    if [ "$MODE" = "--check" ]; then
        if ! grep -Eq "$expect" "$file"; then
            echo "DRIFT: $label in $file (expected /$expect/)"
            fail=1
        fi
    else
        sed -i.bak -e "$sedexpr" "$file" && rm -f "$file.bak"
        grep -Eq "$expect" "$file" || { echo "ERROR: failed to set $label in $file"; exit 1; }
    fi
}

# Table-form pins, in both workspaces. `systemprompt` itself is required in
# each; the others are moved wherever they are declared. tests/ is excluded
# from the root workspace and carries its own copies — nothing else rewrites
# them, and a stale pin there silently disables that workspace's patch.
CRATES="systemprompt systemprompt-security systemprompt-users systemprompt-content systemprompt-marketplace systemprompt-extension systemprompt-api systemprompt-evaluation"
for manifest in Cargo.toml tests/Cargo.toml; do
    for crate in $CRATES; do
        if [ "$crate" != systemprompt ] && ! grep -Eq "^$crate = \\{ version = \"" "$manifest"; then
            continue
        fi
        check_or_apply "$manifest" \
            "s|^$crate = { version = \"[0-9.]*\"|$crate = { version = \"$VERSION\"|" \
            "^$crate = \\{ version = \"$VERSION\"" \
            "$crate core pin"
    done
done

# Residual sweep: any core pin in any manifest that the rules above do not
# already move. A pin added to a new crate would otherwise sit stale forever,
# because no gate distinguishes a forgotten pin from a deliberate one.
#
# Both spellings count. A bare-string pin (`systemprompt-models = "0.43.0"`) is
# as load-bearing as the table form, and sweeping only the table form let a
# stale one sit behind an active patch until the patch came off and the
# lockfile quietly resolved two versions of the same core crate.
# .vendor/ is where CI checks core out (a copy of core itself), not our pins.
stale=$(grep -rnE '^systemprompt[a-z-]* = ("[0-9]|\{ version = ")' --include=Cargo.toml . \
    | grep -v '/target/' | grep -v '/\.vendor/' \
    | grep -vE "= \"$VERSION\"|version = \"$VERSION\"" || true)
if [ -n "$stale" ]; then
    echo "DRIFT: core pins not on $VERSION and not covered by this script:"
    echo "$stale"
    fail=1
    [ "$MODE" = "--check" ] || exit 1
fi

# bridge/CORE_REF names the core CI checks out beside this repo. With patches
# active it is a core SHA (just core-pin); published-core mode needs the tag.
if ! grep -qE '^\[patch\.crates-io\]' Cargo.toml; then
    if [ "$MODE" = "--check" ]; then
        [ "$(tr -d '[:space:]' < bridge/CORE_REF)" = "v$VERSION" ] || {
            echo "DRIFT: bridge/CORE_REF is $(tr -d '[:space:]' < bridge/CORE_REF), expected v$VERSION"
            fail=1
        }
    else
        printf 'v%s\n' "$VERSION" > bridge/CORE_REF
    fi
fi

if [ "$MODE" = "--check" ]; then
    [ "$fail" -eq 0 ] && echo "core sync OK: every core pin on $VERSION" || exit 1
else
    echo "core sync applied: $VERSION"
fi
