#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
sha=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
state_dir="$(mktemp -d)"
trap 'rm -rf "$state_dir"' EXIT
export sha state_dir
mock_gh() {
    if [[ "$*" == *'/jobs?'* ]]; then
        if [ "$case_name" = aggregate_failed ]; then
            echo '{"jobs":[{"name":"Gates passed","status":"completed","conclusion":"failure"}]}'
        elif [ "$case_name" = aggregate_missing ]; then
            echo '{"jobs":[]}'
        else
            echo '{"jobs":[{"name":"Gates passed","status":"completed","conclusion":"success"}]}'
        fi
    else
        local run_sha="$sha" branch=next event=push status=completed conclusion=success attempt=1 id=42
        if [ "$case_name" = rerun_race ] || [ "$case_name" = new_run_race ]; then
            if [ -f "$state_dir/$case_name" ]; then
                if [ "$case_name" = rerun_race ]; then attempt=2; else id=43; fi
            else
                touch "$state_dir/$case_name"
            fi
        fi
        case "$case_name" in
            missing) echo '{"workflow_runs":[]}'; return ;;
            stale) run_sha=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb ;;
            wrong_branch) branch=main ;;
            dispatch) event=workflow_dispatch ;;
            pending) status=in_progress; conclusion=null ;;
            cancelled) conclusion=cancelled ;;
            failed) conclusion=failure ;;
        esac
        jq -n --arg sha "$run_sha" --arg branch "$branch" --arg event "$event" --arg status "$status" --arg conclusion "$conclusion" --argjson attempt "$attempt" --argjson id "$id" \
            '{workflow_runs:[{id:$id,run_attempt:$attempt,head_sha:$sha,head_branch:$branch,event:$event,status:$status,conclusion:$conclusion}]}'
    fi
}
gh() { mock_gh "$@"; }
export -f gh mock_gh
for case_name in success missing stale wrong_branch dispatch pending cancelled failed aggregate_failed aggregate_missing rerun_race new_run_race; do
    export case_name
    expected=1
    [ "$case_name" != success ] || expected=0
    status=0
    bash scripts/check-gates-green.sh "$sha" >/dev/null 2>&1 || status=$?
    [ "$status" -eq "$expected" ] || { echo "$case_name: expected $expected, got $status" >&2; exit 1; }
    echo "$case_name: PASS"
done
