#!/usr/bin/env bash
# Sync every version pin in the repo to a single release version.
#
#   scripts/sync-release-version.sh 0.21.0          # apply
#   scripts/sync-release-version.sh 0.21.0 --check  # verify only (CI guard)
#
# Covered pins:
#   Cargo.toml            workspace version + systemprompt/-security core pins
#   helm/gateway/Chart.yaml  appVersion + artifacthub images annotation
#                            (chart `version:` is bumped separately on apply)
#   deploy/casaos/docker-compose.yml                exact image tag
#   deploy/digitalocean/files/opt/systemprompt/docker-compose.yml  exact image tag
#   deploy/digitalocean/marketplace-image.pkr.hcl   image_version default
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

IMAGE="ghcr.io/systempromptio/systemprompt-template"
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

# Helm chart — appVersion + images annotation.
check_or_apply helm/gateway/Chart.yaml \
    "s|^appVersion: \"[0-9.]*\"|appVersion: \"$VERSION\"|" \
    "^appVersion: \"$VERSION\"" \
    "chart appVersion"
check_or_apply helm/gateway/Chart.yaml \
    "s|image: $IMAGE:[0-9.]*|image: $IMAGE:$VERSION|" \
    "image: $IMAGE:$VERSION" \
    "artifacthub images annotation"

# Exact-pin deploy files.
check_or_apply deploy/casaos/docker-compose.yml \
    "s|image: $IMAGE:[0-9.]*|image: $IMAGE:$VERSION|" \
    "image: $IMAGE:$VERSION" \
    "CasaOS image pin"
check_or_apply deploy/digitalocean/files/opt/systemprompt/docker-compose.yml \
    "s|image: $IMAGE:[0-9.]*|image: $IMAGE:$VERSION|" \
    "image: $IMAGE:$VERSION" \
    "DigitalOcean image pin"
check_or_apply deploy/digitalocean/marketplace-image.pkr.hcl \
    "s|default = \"[0-9.]*\"|default = \"$VERSION\"|" \
    "default = \"$VERSION\"" \
    "Packer image_version default"

if [ "$MODE" = "--check" ]; then
    [ "$fail" -eq 0 ] && echo "version sync OK: everything pinned to $VERSION" || exit 1
else
    # Chart version: bump minor once per release (apply mode only, idempotent
    # via marker check — skip if the changelog already mentions this version).
    if ! grep -q "appVersion to $VERSION" helm/gateway/Chart.yaml; then
        chart_ver=$(sed -n 's/^version: \([0-9.]*\)$/\1/p' helm/gateway/Chart.yaml)
        IFS=. read -r cmaj cmin cpatch <<EOV
$chart_ver
EOV
        new_chart="$cmaj.$((cmin + 1)).0"
        sed -i.bak "s|^version: $chart_ver\$|version: $new_chart|" helm/gateway/Chart.yaml && rm -f helm/gateway/Chart.yaml.bak
        # Prepend a changelog entry inside the artifacthub.io/changes block.
        sed -i.bak "/^  artifacthub.io\/changes: |/a\\
    - kind: changed\\
      description: Bumped appVersion to $VERSION (core + gateway version alignment)." helm/gateway/Chart.yaml && rm -f helm/gateway/Chart.yaml.bak
        echo "chart version: $chart_ver -> $new_chart"
    fi
    echo "version sync applied: $VERSION"
fi
