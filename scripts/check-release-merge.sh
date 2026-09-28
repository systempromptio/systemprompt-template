#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
repo="${RELEASE_REPO:-systempromptio/systemprompt-template}"
merge="${1:?usage: check-release-merge.sh <main-sha>}"
die() { echo "release-proof: $*" >&2; exit 1; }
[[ "$merge" =~ ^[0-9a-f]{40}$ ]] || die 'expected full merge SHA'
[ "$(git rev-parse HEAD)" = "$merge" ] || die 'checkout differs from release commit'
parents="$(git show -s --format=%P "$merge")"
read -r base head extra <<<"$parents"
[ -n "$base" ] && [ -n "$head" ] && [ -z "$extra" ] || die 'release is not a two-parent promotion merge'
[ "$(git rev-parse "$merge^{tree}")" = "$(git rev-parse "$head^{tree}")" ] || die 'merged tree differs from candidate'
prs="$(gh api --paginate "repos/$repo/commits/$merge/pulls?per_page=100")"
pr="$(jq -sce --arg merge "$merge" --arg head "$head" --arg repo "$repo" '
    [ .[][] | select(.merged_at != null and .merge_commit_sha == $merge and .base.ref == "main" and .head.sha == $head and .head.repo.full_name == $repo) ] |
    if length == 1 then .[0] else error("expected exactly one merged promotion PR") end' <<<"$prs")" || die 'missing unambiguous merged promotion PR'
ref="$(jq -r .head.ref <<<"$pr")"
[[ "$ref" =~ ^promote/([0-9]+\.[0-9]+\.[0-9]+)/([0-9a-f]{40})/([0-9a-f]{40})$ ]] || die 'PR is not a frozen promotion'
[ "${BASH_REMATCH[2]}" = "$base" ] && [ "${BASH_REMATCH[3]}" = "$head" ] || die 'merge parents differ from frozen promotion'
version="${BASH_REMATCH[1]}"
scripts/sync-release-version.sh "$version" --check
scripts/sync-core-version.sh --check
bash scripts/check-gates-green.sh "$head"
identity="$(bash scripts/check-promotion-green.sh "$head" "$ref")" || die 'promotion proof did not succeed'
IFS=$'\t' read -r run_id attempt <<<"$identity"
echo "Release merge $merge verified against frozen candidate $head, base $base, promotion run $run_id attempt $attempt"
