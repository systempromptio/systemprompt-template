#!/usr/bin/env bash
# Sync every version pin in the repo to a single release version.
#
#   scripts/sync-release-version.sh 0.21.0          # apply
#   scripts/sync-release-version.sh 0.21.0 --check  # verify only (CI guard)
#
# The template's own version and the core release it depends on are usually the
# same number, and CORE_VERSION defaults to the version given. They diverge on a
# template-only patch release — 0.37.1 shipping against an unchanged core
# 0.37.0 — where pinning the core crates to the template's version would name a
# release that was never published:
#
#   CORE_VERSION=0.37.0 scripts/sync-release-version.sh 0.37.1
#
# In --check mode CORE_VERSION defaults instead to the pin already in the root
# Cargo.toml, so the guard asserts what it is actually there to assert: that
# every core pin agrees with every other, and that the product version is the
# one being released.
#
# Lockstep: the release and the core it ships share MAJOR.MINOR. --check
# refuses a release whose X.Y differs from the core pin's, so a template
# 0.62.0 can never go out on core 0.61.x. A template-only patch (0.37.1 on
# core 0.37.0) is the one sanctioned divergence and passes. Core pins and
# bridge/CORE_REF beyond the two listed below are scripts/sync-core-version.sh's.
#
# Between releases core may be pinned by git rev to an unreleased branch. That
# tree cannot ship — scripts/release.sh refuses a git-pinned or patched core
# before it reaches this check — so the workspace version still names the last
# release and lockstep has no published core to compare it with. --check then
# reports the git pin and asserts every other pin; lockstep is judged on the
# published-core tree the release boundary produces.
#
# Covered pins:
#   Cargo.toml            workspace version + systemprompt/-extension
#                         core pins (extensions/web inherits -extension)
#   tests/Cargo.toml      systemprompt/-security pins (separate workspace)
#   extensions/harness/Cargo.toml  workspace version (separate workspace)
#   extensions/mobile/    the phone apps' version: shared/toolchain.env,
#                         the Android version catalog, the Xcode project,
#                         and the binding workspace (core/Cargo.toml and
#                         its own package's entry in core/Cargo.lock)
#   helm/gateway/Chart.yaml  appVersion + artifacthub images annotation
#                            (chart `version:` is bumped separately on apply)
#   deploy/casaos/docker-compose.yml                exact image tag
#   deploy/digitalocean/files/opt/systemprompt/docker-compose.yml  exact image tag
#   deploy/digitalocean/marketplace-image.pkr.hcl   image_version default
#   docs/install/*.md, deploy/*/*.md  every 0.x.y literal (apply only; lines
#                            ending `<!-- pinned-release -->` are historical
#                            and left alone; scripts/check-docs-version.sh is
#                            the check)
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

if [ "$MODE" = "--check" ]; then
    CORE_VERSION="${CORE_VERSION:-$(sed -n 's|^systemprompt = { version = "\([0-9.]*\)".*|\1|p' Cargo.toml | head -1)}"
    [ -n "$CORE_VERSION" ] || { echo "ERROR: no systemprompt core pin found in Cargo.toml"; exit 1; }
else
    CORE_VERSION="${CORE_VERSION:-$VERSION}"
fi
case "$CORE_VERSION" in
  *[!0-9.]*|*..*|.*|*.) echo "ERROR: CORE_VERSION '$CORE_VERSION' is not a plain semver (X.Y.Z)"; exit 1 ;;
esac
core_git_pinned=0
if sed -n '/^systemprompt = {/p' Cargo.toml | head -1 | command grep -qF 'git = "https://github.com/systempromptio/systemprompt-core"'; then
    core_git_pinned=1
fi
if [ "$MODE" = "--check" ] && [ "$core_git_pinned" -eq 1 ]; then
    echo "core is pinned by git rev (unreleased $CORE_VERSION): lockstep is asserted on the release tree"
elif [ "${CORE_VERSION%.*}" != "$MAJ.$MIN" ]; then
    echo "DRIFT: release $VERSION is not in lockstep with core $CORE_VERSION (MAJOR.MINOR must match)"
    fail=1
fi

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

# The desktop shell is a separate workspace and does not inherit the root version.
check_or_apply extensions/harness/Cargo.toml \
    "s|^version = \"[0-9.]*\"|version = \"$VERSION\"|" \
    "^version = \"$VERSION\"" \
    "shell workspace version"

# The phone apps' binding library is a separate workspace too. Its lockfile
# names its own package's version, so that entry moves with the manifest.
check_or_apply extensions/mobile/core/Cargo.toml \
    "s|^version = \"[0-9.]*\"|version = \"$VERSION\"|" \
    "^version = \"$VERSION\"" \
    "mobile binding workspace version"
check_or_apply extensions/mobile/core/Cargo.lock \
    "/^name = \"systemprompt-mobile-app\"$/{n;s|^version = \"[0-9.]*\"|version = \"$VERSION\"|;}" \
    "^version = \"$VERSION\"" \
    "mobile binding workspace lockfile entry"

# The phone apps themselves are not cargo packages; their version is written
# in three places that scripts/mobile/toolchain.sh holds to each other.
check_or_apply extensions/mobile/shared/toolchain.env \
    "s|^MOBILE_APP_VERSION=[0-9.]*|MOBILE_APP_VERSION=$VERSION|" \
    "^MOBILE_APP_VERSION=$VERSION\$" \
    "mobile app version"
check_or_apply extensions/mobile/android/gradle/libs.versions.toml \
    "s|^appVersionName = \"[0-9.]*\"|appVersionName = \"$VERSION\"|" \
    "^appVersionName = \"$VERSION\"" \
    "Android app versionName"
check_or_apply extensions/mobile/ios/Mobile.xcodeproj/project.pbxproj \
    "s|MARKETING_VERSION = [0-9.]*;|MARKETING_VERSION = $VERSION;|" \
    "MARKETING_VERSION = $VERSION;" \
    "iOS app marketing version"

# Cargo.toml — core crate pins.
check_or_apply Cargo.toml \
    "s|^systemprompt = { version = \"[0-9.]*\"|systemprompt = { version = \"$CORE_VERSION\"|" \
    "^systemprompt = \\{ version = \"$CORE_VERSION\"" \
    "systemprompt core pin"
check_or_apply Cargo.toml \
    "s|^systemprompt-extension = { version = \"[0-9.]*\"|systemprompt-extension = { version = \"$CORE_VERSION\"|" \
    "^systemprompt-extension = \\{ version = \"$CORE_VERSION\"" \
    "systemprompt-extension core pin"

# tests/ is a separate workspace and does not inherit the root pins.
check_or_apply tests/Cargo.toml \
    "s|^systemprompt = { version = \"[0-9.]*\"|systemprompt = { version = \"$CORE_VERSION\"|" \
    "^systemprompt = \\{ version = \"$CORE_VERSION\"" \
    "test workspace systemprompt pin"
check_or_apply tests/Cargo.toml \
    "s|^systemprompt-security = { version = \"[0-9.]*\"|systemprompt-security = { version = \"$CORE_VERSION\"|" \
    "^systemprompt-security = \\{ version = \"$CORE_VERSION\"" \
    "test workspace systemprompt-security pin"

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
check_or_apply deploy/casaos/docker-compose.yml \
    "s|^  version: \"[0-9.]*\"|  version: \"$VERSION\"|" \
    "^  version: \"$VERSION\"" \
    "CasaOS x-casaos version"
check_or_apply deploy/digitalocean/files/opt/systemprompt/docker-compose.yml \
    "s|image: $IMAGE:[0-9.]*|image: $IMAGE:$VERSION|" \
    "image: $IMAGE:$VERSION" \
    "DigitalOcean image pin"
check_or_apply deploy/digitalocean/marketplace-image.pkr.hcl \
    "s|default = \"[0-9.]*\"|default = \"$VERSION\"|" \
    "default = \"$VERSION\"" \
    "Packer image_version default"

if [ "$MODE" != "--check" ]; then
    for doc in docs/install/*.md deploy/*/*.md; do
        [ -f "$doc" ] || continue
        sed -i.bak -E "/<!-- pinned-release -->\$/!s/(^|[^0-9.])0\.[0-9]+\.[0-9]+([^0-9.]|\$)/\1$VERSION\2/g" "$doc" && rm -f "$doc.bak"
    done
fi

if [ "$MODE" = "--check" ]; then
    [ "$fail" -eq 0 ] && echo "version sync OK: everything pinned to $VERSION" || exit 1
else
    # Chart version: bumped once per release (apply mode only, idempotent via
    # marker check — skip if the changelog already mentions this version). The
    # chart tracks the shape of the release it carries: a patch release of the
    # gateway bumps the chart's patch, anything else its minor. Bumping the
    # minor for a patch would claim a chart change that did not happen.
    if ! grep -q "appVersion to $VERSION" helm/gateway/Chart.yaml; then
        chart_ver=$(sed -n 's/^version: \([0-9.]*\)$/\1/p' helm/gateway/Chart.yaml)
        IFS=. read -r cmaj cmin cpatch <<EOV
$chart_ver
EOV
        if [ "$PATCH" = "0" ]; then
            new_chart="$cmaj.$((cmin + 1)).0"
        else
            new_chart="$cmaj.$cmin.$((cpatch + 1))"
        fi
        sed -i.bak "s|^version: $chart_ver\$|version: $new_chart|" helm/gateway/Chart.yaml && rm -f helm/gateway/Chart.yaml.bak
        # Why: sed's `a\` continuation drops the trailing newline under BSD sed,
        # which silently welded the 0.34.0 entry onto the next list item and left
        # invalid YAML in the published annotation.
        VERSION="$VERSION" python3 - helm/gateway/Chart.yaml <<'PYEOF'
import os, sys
path = sys.argv[1]
marker = "  artifacthub.io/changes: |\n"
entry = (
    "    - kind: changed\n"
    "      description: Bumped appVersion to %s (core + gateway version alignment).\n"
    % os.environ["VERSION"]
)
text = open(path).read()
if text.count(marker) != 1:
    sys.exit("Chart.yaml: expected exactly one artifacthub.io/changes block")
open(path, "w").write(text.replace(marker, marker + entry, 1))
PYEOF
        echo "chart version: $chart_ver -> $new_chart"
    fi
    echo "version sync applied: $VERSION"
fi
