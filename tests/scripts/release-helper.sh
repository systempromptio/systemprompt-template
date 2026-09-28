#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
head=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
base=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
merge=cccccccccccccccccccccccccccccccccccccccc
version="$(sed -n 's/^version = "\([0-9.]*\)"/\1/p' Cargo.toml | head -1)"
state_dir="$(mktemp -d)"
trap 'rm -rf "$state_dir"' EXIT
export head base merge version state_dir
mock_git() {
    case "$*" in
        'status --porcelain'|'branch --show-current'|"merge-base --is-ancestor $base $head") ;;
        "merge-base --is-ancestor $head origin/main") return 1 ;;
        "ls-remote origin refs/tags/v$version") ;;
        "tag v$version $merge") touch "$state_dir/tagged-locally" ;;
        "push origin refs/tags/v$version")
            [ -f "$state_dir/merged" ] && [ -f "$state_dir/tagged-locally" ] || { echo 'tag pushed before the verified merge' >&2; return 1; }
            touch "$state_dir/tag-pushed" ;;
        'rev-parse HEAD'|'rev-parse origin/next') echo "$head" ;;
        'rev-parse origin/main')
            if [ -f "$state_dir/merged" ]; then echo "$merge"; else echo "$base"; fi ;;
        "rev-parse $head^{tree}"|"rev-parse $merge^{tree}") echo candidate-tree ;;
        'rev-parse FETCH_HEAD^{tree}') cat "$state_dir/fetch-tree" ;;
        "rev-parse $merge^1") echo "$base" ;;
        "rev-parse $merge^2") echo "$head" ;;
        'fetch origin '* )
            if [ "$3" = refs/pull/2/merge ]; then echo candidate-tree; else echo previous-main-tree; fi > "$state_dir/fetch-tree" ;;
        'ls-remote origin '* ) printf '%s\t%s\n' "$head" "$3" ;;
        *) echo "unexpected git $*" >&2; return 1 ;;
    esac
}
mock_gh() {
    local ref="promote/$version/$base/$head"
    case "$*" in
        'pr list '*) echo 2 ;;
        'pr view '*) jq -n --arg head "$head" --arg base "$base" '{headRefOid:$head,baseRefOid:$base,mergeable:"MERGEABLE"}' ;;
        'pr merge '*) touch "$state_dir/merged" ;;
        *'/jobs?'*)
            if [[ "$*" == *'/runs/84/'* ]]; then
                echo '{"jobs":[{"name":"Gates passed","status":"completed","conclusion":"success"},{"name":"Verify frozen promotion","status":"completed","conclusion":"success"}]}'
            else echo '{"jobs":[{"name":"Gates passed","status":"completed","conclusion":"success"}]}'; fi ;;
        *)
            local event=push branch=next id=42
            if [[ "$*" == *'event=pull_request'* ]]; then event=pull_request branch="$ref" id=84; fi
            jq -n --arg sha "$head" --arg event "$event" --arg branch "$branch" --argjson id "$id" \
                '{workflow_runs:[{id:$id,run_attempt:1,head_sha:$sha,head_branch:$branch,event:$event,status:"completed",conclusion:"success"}]}' ;;
    esac
}
git() { mock_git "$@"; }
gh() { mock_gh "$@"; }
# `next` normally carries active sibling-core patches, while release.sh quite
# correctly accepts only the published-core candidate produced at the release
# boundary. This harness mocks that frozen candidate's git/GitHub state; mock
# only the patch-presence probe as well, rather than requiring the live branch
# under test to be in main's published-core shape.
grep() {
    if [ "$#" -eq 4 ] \
        && [ "$1" = -q ] \
        && [ "$2" = '^\[patch\.crates-io\]' ] \
        && [ "$3" = Cargo.toml ] \
        && [ "$4" = tests/Cargo.toml ]; then
        return 1
    fi
    # The changelog heading is written as part of the release commit; the
    # harness is not that commit, so mock the probe the same way.
    if [ "$#" -eq 3 ] && [ "$1" = -qE ] && [ "$3" = CHANGELOG.md ]; then
        return 0
    fi
    command grep "$@"
}
# The schema ladder has its own gate (check-schema-baseline.sh in lint-gates);
# this harness proves the promotion protocol, not the rung for the version.
bash() {
    if [ "${1:-}" = scripts/check-schema-baseline.sh ]; then return 0; fi
    command bash "$@"
}
export -f git gh grep bash mock_git mock_gh
bash scripts/release.sh "$version"
[ -f "$state_dir/merged" ] || { echo 'promotion helper never merged the verified candidate' >&2; exit 1; }
[ -f "$state_dir/tag-pushed" ] || { echo 'promotion helper never tagged the release merge' >&2; exit 1; }
echo 'promotion fetches the exact PR merge tree and tags the merge: PASS'
