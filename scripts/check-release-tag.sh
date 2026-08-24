#!/usr/bin/env bash
# Every released version must carry its tag — because the image depends on it.
#
# The chart publishes from the push to main (helm.yml), but the versioned image
# is published only by release-gateway.yml, which runs on a `v*` tag. Skip the
# tag and the chart still goes out advertising
# `ghcr.io/systempromptio/systemprompt-template:<appVersion>`, an image that
# will never exist. Nothing fails at the time; what arrives later is a
# vulnerability scanner mailing "image not found" for that chart version, on
# every scan, forever — 0.32.0, 0.34.0 and 0.35.0 each reached that state, and
# each is exactly a release whose tag was never pushed.
#
# The invariant, local and network-free: a version in CHANGELOG.md older than
# the workspace version must have a `v<version>` tag. Once the manifest has
# moved past a version, that version shipped, so its tag has to exist. The
# version in the manifest is exempt — its changelog entry and chart bump are
# written before the tag, so requiring it would fail every release in progress.
set -uo pipefail

cd "$(dirname "$0")/.."

CHANGELOG="CHANGELOG.md"
MANIFEST="Cargo.toml"

# Tagging became consistent at 0.27.0; earlier entries predate the convention.
FLOOR="0.27.0"

[ -f "$CHANGELOG" ] || { echo "check-release-tag: no $CHANGELOG" >&2; exit 2; }

CURRENT=$(awk '/^\[workspace\.package\]/{p=1;next}/^\[/{p=0}p&&/^version[[:space:]]*=/{gsub(/[[:space:]"]/,"");sub(/^version=/,"");print;exit}' "$MANIFEST")
[ -n "$CURRENT" ] || { echo "check-release-tag: could not read workspace version" >&2; exit 2; }

# A shallow CI checkout carries no tags, and fetching them all just to read
# their names would drag the whole history down. Only names are needed.
TAGS=$(git tag 2>/dev/null)
if [ -z "$TAGS" ]; then
    TAGS=$(git ls-remote --tags --refs origin 2>/dev/null | sed 's|.*refs/tags/||')
    [ -n "$TAGS" ] || { echo "check-release-tag: no tags locally and none readable from origin" >&2; exit 2; }
fi

has_tag() { printf '%s\n' "$TAGS" | grep -qxF "$1"; }
older_than_current() {
    [ "$1" != "$CURRENT" ] && [ "$(printf '%s\n%s\n' "$1" "$CURRENT" | sort -V | head -1)" = "$1" ]
}
at_or_above_floor() { [ "$(printf '%s\n%s\n' "$1" "$FLOOR" | sort -V | head -1)" = "$FLOOR" ]; }

# A heading marked "never released" is exempt: the version exists publicly as a
# chart but has no installable image, and no tag can honestly be created for it.
# The count is printed rather than passed over in silence, so the exemption stays
# visible instead of becoming somewhere to hide a real gap.
SKIPPED=$(grep -cE '^## \[?[0-9]+\.[0-9]+\.[0-9]+\]?.*never released' "$CHANGELOG" || true)

MISSING=""
CHECKED=0
while IFS= read -r version; do
    older_than_current "$version" || continue
    at_or_above_floor "$version" || continue
    grep -qE "^## \[?${version}\]?.*never released" "$CHANGELOG" && continue
    CHECKED=$((CHECKED + 1))
    has_tag "v${version}" && continue
    MISSING="${MISSING}  v${version} — released (CHANGELOG) but no tag, so no image was ever built"$'\n'
done < <(grep -oE '^## \[?[0-9]+\.[0-9]+\.[0-9]+\]?' "$CHANGELOG" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | sort -u -V)

if [ -n "$MISSING" ]; then
    echo "check-release-tag: released versions with no tag:"
    echo ""
    printf '%s' "$MISSING"
    echo ""
    echo "release-gateway.yml is the only producer of the versioned image, and it"
    echo "runs on a v* tag. Push the tag for the release commit:"
    echo "    git tag v<version> <commit> && git push origin v<version>"
    exit 1
fi

echo "check-release-tag: OK ($CHECKED released version(s) >= $FLOOR tagged; $CURRENT in flight; $SKIPPED marked never released)"
