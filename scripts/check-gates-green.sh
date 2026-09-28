#!/usr/bin/env bash
set -euo pipefail
sha="${1:?usage: check-gates-green.sh <full-sha>}"
repo="${RELEASE_REPO:-systempromptio/systemprompt-template}"
[[ "$sha" =~ ^[0-9a-f]{40}$ ]] || { echo 'ERROR: expected a full commit SHA' >&2; exit 1; }
query="repos/$repo/actions/workflows/gates.yml/runs?event=push&branch=next&head_sha=$sha&per_page=1"
run="$(gh api "$query")"
proof="$(jq -er --arg sha "$sha" '.workflow_runs[0] | select(.head_sha == $sha and .head_branch == "next" and .event == "push" and .status == "completed" and .conclusion == "success") | [.id,.run_attempt] | @tsv' <<<"$run")" || {
    echo "ERROR: latest next-push Gates run on $sha is missing, pending, or unsuccessful" >&2; exit 1;
}
IFS=$'\t' read -r run_id attempt <<<"$proof"
[[ "$run_id" =~ ^[0-9]+$ && "$attempt" =~ ^[1-9][0-9]*$ ]] || { echo 'ERROR: missing run identity or attempt' >&2; exit 1; }
gh api --paginate "repos/$repo/actions/runs/$run_id/attempts/$attempt/jobs?per_page=100" | jq -se '
    [.[].jobs[] | select(.name == "Gates passed")] |
    length == 1 and all(.[]; .status == "completed" and .conclusion == "success")' >/dev/null || {
    echo "ERROR: Gates run $run_id attempt $attempt has no successful Gates passed aggregate" >&2; exit 1;
}
gh api "$query" | jq -e --arg sha "$sha" --argjson id "$run_id" --argjson attempt "$attempt" '
    .workflow_runs[0] | .id == $id and .run_attempt == $attempt and .head_sha == $sha and .head_branch == "next" and .event == "push" and .status == "completed" and .conclusion == "success"' >/dev/null || {
    echo 'ERROR: latest Gates proof changed during verification' >&2; exit 1;
}
echo "Gates passed on $sha (next push run $run_id attempt $attempt)"
