#!/usr/bin/env bash
# Promote the exact green next-push candidate through a frozen PR onto main,
# then tag the merge so release-gateway.yml publishes it. `just release X.Y.Z`.
#
# Run it twice. The first run verifies the candidate (clean tree, published
# core, lockstep pins, changelog entry, schema ladder, a green `Gates passed`
# on the exact next-push run), pushes the frozen ref
# promote/<version>/<main>/<candidate> and opens the PR onto main. Gates.yml
# then runs its lightweight "Verify frozen promotion" proof on that PR. The
# second run finds the proof, re-checks that main has not moved and that the
# proposed merge tree is the proven tree, merges, and pushes the vX.Y.Z tag at
# the merge commit.
#
# Why this script pushes the tag rather than a workflow: release-gateway.yml
# (tarballs, image, chart, Homebrew, smoke) runs on a `v*` tag push, and a tag
# pushed by a workflow's GITHUB_TOKEN never starts another workflow. Its first
# job re-verifies the merge (scripts/check-release-merge.sh) before anything
# is built. If the tag push fails after the merge, re-running this command
# resumes at the tag step.
set -euo pipefail
cd "$(dirname "$0")/.."
version="${1:?usage: release.sh X.Y.Z}"
repo="${RELEASE_REPO:-systempromptio/systemprompt-template}"
tag="v$version"
die() { echo "release: $*" >&2; exit 1; }

# Tag the merge commit, once. An existing tag must already name this merge.
push_tag() {
    local merge="$1" existing
    existing="$(git ls-remote origin "refs/tags/$tag" | cut -f1)"
    if [ -z "$existing" ]; then
        git tag "$tag" "$merge"
        git push origin "refs/tags/$tag"
    else
        [ "$existing" = "$merge" ] || die "tag $tag already exists at $existing, not at the release merge $merge"
    fi
    echo "main -> $merge tagged $tag; watch release-gateway.yml (tarballs, image, chart, Homebrew, smoke)"
}

[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die 'expected X.Y.Z'
[ -z "$(git status --porcelain)" ] || die 'working tree is not clean'
branch="$(git branch --show-current)"
[ -z "$branch" ] || [ "$branch" = next ] || die 'use next or a detached release worktree'
! grep -q '^\[patch\.crates-io\]' Cargo.toml tests/Cargo.toml || die 'published releases cannot use local core patches'
scripts/sync-release-version.sh "$version" --check
scripts/sync-core-version.sh --check
bash scripts/check-core-ref.sh
bash scripts/check-schema-baseline.sh
# check-release-tag.sh requires every released CHANGELOG version to carry its
# tag, so the release needs its heading before the tag exists.
grep -qE "^## \[?$version\]?" CHANGELOG.md || die "CHANGELOG.md has no '## [$version]' heading"
git fetch origin main next
sha="$(git rev-parse HEAD)"
[ "$sha" = "$(git rev-parse origin/next)" ] || die 'HEAD differs from origin/next'

# Resume: the candidate already merged (a previous run's tag push failed).
if git merge-base --is-ancestor "$sha" origin/main; then
    merge="$(git rev-parse origin/main)"
    [ "$(git rev-parse "$merge^2" 2>/dev/null)" = "$sha" ] || die 'main contains the candidate but its tip is not the promotion merge'
    push_tag "$merge"
    exit 0
fi

base="$(git rev-parse origin/main)"
[ "$sha" != "$base" ] || die 'candidate is already main'
git merge-base --is-ancestor "$base" "$sha" || die 'main is not an ancestor of candidate'
bash scripts/check-gates-green.sh "$sha"
ref="promote/$version/$base/$sha"
remote="$(git ls-remote origin "refs/heads/$ref" | cut -f1)"
if [ -z "$remote" ]; then
    git push origin "$sha:refs/heads/$ref"
else
    [ "$remote" = "$sha" ] || die 'promotion ref differs from frozen candidate'
fi
pr="$(gh pr list -R "$repo" --base main --head "$ref" --state open --json number --jq '.[0].number // empty')"
if [ -z "$pr" ]; then
    body="$(mktemp)"
    trap 'rm -f "$body"' EXIT
    printf 'Release %s against published core crates.\n\nFrozen candidate: `%s`\nMain base: `%s`\n\nThe exact next-push Gates run proves the candidate; promotion verifies that proof and the merge tree without repeating the matrix. After the merge, `just release %s` pushes the %s tag that starts release-gateway.yml.\n' "$version" "$sha" "$base" "$version" "$tag" > "$body"
    gh pr create -R "$repo" --base main --head "$ref" --title "Release $version" --body-file "$body"
    pr="$(gh pr list -R "$repo" --base main --head "$ref" --state open --json number --jq '.[0].number')"
fi
state="$(gh pr view "$pr" -R "$repo" --json headRefOid,baseRefOid,mergeable)"
jq -e --arg sha "$sha" --arg base "$base" '.headRefOid == $sha and .baseRefOid == $base' <<<"$state" >/dev/null || die 'PR head/base moved'
if proof="$(bash scripts/check-promotion-green.sh "$sha" "$ref")"; then
    IFS=$'\t' read -r run_id attempt <<<"$proof"
    echo "Promotion PR #$pr verified by run $run_id attempt $attempt"
else
    status=$?
    if [ "$status" -ne 2 ]; then die 'promotion proof failed; inspect the exact PR run'; fi
    echo "Promotion PR #$pr is open; wait for its Gates run (Verify frozen promotion), then repeat: just release $version"
    exit 0
fi
[ "$(jq -r .mergeable <<<"$state")" = MERGEABLE ] || die 'PR is not mergeable'
git fetch origin main
git fetch origin "refs/pull/$pr/merge"
[ "$(git rev-parse origin/main)" = "$base" ] || die 'main moved after proof'
[ "$(git rev-parse "FETCH_HEAD^{tree}")" = "$(git rev-parse "$sha^{tree}")" ] || die 'proposed merge tree differs from proven candidate'
bash scripts/check-gates-green.sh "$sha"
gh pr merge "$pr" -R "$repo" --merge --match-head-commit "$sha"
git fetch origin main
merge="$(git rev-parse origin/main)"
[ "$(git rev-parse "$merge^1")" = "$base" ] || die 'merged main base changed'
[ "$(git rev-parse "$merge^2")" = "$sha" ] || die 'merged candidate changed'
[ "$(git rev-parse "$merge^{tree}")" = "$(git rev-parse "$sha^{tree}")" ] || die 'merged tree differs from proven candidate'
push_tag "$merge"
