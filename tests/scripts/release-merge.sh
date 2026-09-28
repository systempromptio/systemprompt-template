#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
merge=cccccccccccccccccccccccccccccccccccccccc
head=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
base=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
state_dir="$(mktemp -d)"
trap 'rm -rf "$state_dir"' EXIT
version="$(sed -n 's/^version = "\([0-9.]*\)"/\1/p' Cargo.toml | head -1)"
export merge head base state_dir version
mock_git() {
    case "$*" in
        'rev-parse HEAD') echo "$merge" ;;
        "show -s --format=%P $merge")
            if [ "$case_name" = single_parent ]; then echo "$head"; else echo "$base $head"; fi ;;
        "rev-parse $merge^{tree}") echo tree ;;
        "rev-parse $head^{tree}")
            if [ "$case_name" = changed_tree ]; then echo changed; else echo tree; fi ;;
        *) echo "unexpected git $*" >&2; return 1 ;;
    esac
}
mock_gh() {
    local ref="promote/$version/$base/$head"
    if [[ "$*" == *'/commits/'* ]]; then
        [ "$case_name" != changed_base ] || ref="promote/$version/$head/$head"
        jq -n --arg merge "$merge" --arg head "$head" --arg ref "$ref" '[{merged_at:"today",merge_commit_sha:$merge,base:{ref:"main"},head:{sha:$head,ref:$ref,repo:{full_name:"systempromptio/systemprompt-template"}}}]'
    elif [[ "$*" == *'/jobs?'* ]]; then
        if [[ "$*" == *'/runs/84/'* ]]; then
            if [ "$case_name" = duplicate_aggregate ]; then
                echo '{"jobs":[{"name":"Gates passed","status":"completed","conclusion":"success"},{"name":"Gates passed","status":"completed","conclusion":"success"}]}'
            elif [ "$case_name" = missing_promotion_job ]; then echo '{"jobs":[]}'; else
                echo '{"jobs":[{"name":"Gates passed","status":"completed","conclusion":"success"},{"name":"Verify frozen promotion","status":"completed","conclusion":"success"}]}'
            fi
        else echo '{"jobs":[{"name":"Gates passed","status":"completed","conclusion":"success"}]}'; fi
    else
        local event=push branch=next id=42 attempt=1 conclusion=success status=completed
        if [[ "$*" == *'event=pull_request'* ]]; then
            event=pull_request branch="$ref" id=84
            [ "$case_name" != pending_promotion ] || status=in_progress
            if [ "$case_name" = missing_promotion ]; then echo '{"workflow_runs":[]}'; return; fi
            [ "$case_name" != red_promotion ] || conclusion=failure
            if [ "$case_name" = promotion_rerun ]; then
                if [ -f "$state_dir/rerun" ]; then attempt=2; else touch "$state_dir/rerun"; fi
            fi
        fi
        jq -n --arg sha "$head" --arg event "$event" --arg branch "$branch" --arg conclusion "$conclusion" --argjson id "$id" --argjson attempt "$attempt" --arg status "$status" \
            '{workflow_runs:[{id:$id,run_attempt:$attempt,head_sha:$sha,head_branch:$branch,event:$event,status:$status,conclusion:$conclusion}]}'
    fi
}
git() { mock_git "$@"; }
gh() { mock_gh "$@"; }
export -f git gh mock_git mock_gh
for case_name in success single_parent changed_tree changed_base missing_promotion_job red_promotion promotion_rerun duplicate_aggregate pending_promotion missing_promotion; do
    export case_name
    expected=1
    [ "$case_name" != success ] || expected=0
    status=0
    bash scripts/check-release-merge.sh "$merge" >/dev/null 2>&1 || status=$?
    [ "$status" -eq "$expected" ] || { echo "$case_name: expected $expected, got $status" >&2; exit 1; }
    echo "$case_name: PASS"
done

for case_name in pending_promotion missing_promotion; do
    export case_name
    status=0
    bash scripts/check-promotion-green.sh "$head" "promote/$version/$base/$head" >/dev/null 2>&1 || status=$?
    [ "$status" -eq 2 ] || { echo "$case_name readiness: expected 2, got $status" >&2; exit 1; }
    echo "$case_name readiness: PASS"
done
