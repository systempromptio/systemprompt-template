#!/usr/bin/env bash
# Sync every version pin in the repo to a single release version.
#
#   scripts/sync-release-version.sh 0.21.0          # apply
#   scripts/sync-release-version.sh 0.21.0 --check  # verify only (CI guard)
#
# Covered pins:
#   Cargo.toml            workspace version + systemprompt/-security/-extension pins
#   tests/Cargo.toml      its own systemprompt/-security pins (separate workspace)
#
# macOS + Linux compatible (no GNU-only sed flags).
set -eu

VERSION="${1:?usage: sync-release-version.sh <version> [--check]}"
MODE="${2:-apply}"
cd "$(dirname "$0")/.."

case "$VERSION" in
  *[!0-9.]*|*..*|.*|*.) echo "ERROR: '$VERSION' is not a plain semver (X.Y.Z)"; exit 1 ;;
esac
IFS=. read -r MAJ MIN PATCH <<EOV
$VERSION
EOV
: "${PATCH:?ERROR: version must have three components}"

fail=0

# file, description, grep pattern that must match post-apply
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

# Cargo.toml — workspace version (first `version =` in [workspace.package]).
check_or_apply Cargo.toml \
    "s|^version = \"[0-9.]*\"|version = \"$VERSION\"|" \
    "^version = \"$VERSION\"" \
    "workspace version"

# Cargo.toml — core crate pins.
check_or_apply Cargo.toml \
    "s|^systemprompt = { version = \"[0-9.]*\"|systemprompt = { version = \"$VERSION\"|" \
    "^systemprompt = \\{ version = \"$VERSION\"" \
    "systemprompt core pin"
check_or_apply Cargo.toml \
    "s|^systemprompt-security = { version = \"[0-9.]*\"|systemprompt-security = { version = \"$VERSION\"|" \
    "^systemprompt-security = \\{ version = \"$VERSION\"" \
    "systemprompt-security core pin"
check_or_apply Cargo.toml \
    "s|^systemprompt-extension = { version = \"[0-9.]*\"|systemprompt-extension = { version = \"$VERSION\"|" \
    "^systemprompt-extension = \\{ version = \"$VERSION\"" \
    "systemprompt-extension core pin"

# tests/Cargo.toml — the test workspace is excluded from the root workspace and
# carries its own copies of the same pins. Nothing else rewrites them, and a
# stale pin here silently disables the test workspace's [patch.crates-io].
check_or_apply tests/Cargo.toml \
    "s|^systemprompt = { version = \"[0-9.]*\"|systemprompt = { version = \"$VERSION\"|" \
    "^systemprompt = \\{ version = \"$VERSION\"" \
    "systemprompt core pin (test workspace)"
check_or_apply tests/Cargo.toml \
    "s|^systemprompt-security = { version = \"[0-9.]*\"|systemprompt-security = { version = \"$VERSION\"|" \
    "^systemprompt-security = \\{ version = \"$VERSION\"" \
    "systemprompt-security core pin (test workspace)"

# Residual sweep: any core pin in any manifest that the rules above do not
# already move. A pin added to a new crate would otherwise sit stale forever,
# because no gate distinguishes a forgotten pin from a deliberate one.
stale=$(grep -rn '^systemprompt[a-z-]* = { version = "' --include=Cargo.toml . \
    | grep -v '/target/' | grep -v "version = \"$VERSION\"" || true)
if [ -n "$stale" ]; then
    echo "DRIFT: core pins not on $VERSION and not covered by this script:"
    echo "$stale"
    fail=1
    [ "$MODE" = "--check" ] || exit 1
fi

if [ "$MODE" = "--check" ]; then
    [ "$fail" -eq 0 ] && echo "version sync OK: everything pinned to $VERSION" || exit 1
else
    echo "version sync applied: $VERSION"
fi
