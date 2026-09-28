#!/usr/bin/env bash
set -euo pipefail
head="${1:?usage: check-promotion-green.sh <head-sha> <promotion-ref>}"
ref="${2:?usage: check-promotion-green.sh <head-sha> <promotion-ref>}"
repo="${RELEASE_REPO:-systempromptio/systemprompt-template}"
die() { echo "promotion-proof: $*" >&2; exit 1; }
[[ "$head" =~ ^[0-9a-f]{40}$ ]] || die 'expected full candidate SHA'
[[ "$ref" =~ ^promote/[0-9]+\.[0-9]+\.[0-9]+/[0-9a-f]{40}/$head$ ]] || die 'promotion ref differs from candidate'
get_proof() {
    gh api -X GET "repos/$repo/actions/workflows/gates.yml/runs" -f event=pull_request -f branch="$ref" -f head_sha="$head" -f per_page=1
}
proof="$(get_proof)"
if jq -e '.workflow_runs | length == 0' <<<"$proof" >/dev/null; then
    echo 'promotion-proof: no promotion run yet' >&2
    exit 2
fi
jq -e --arg sha "$head" --arg branch "$ref" '.workflow_runs[0] | .event == "pull_request" and .head_branch == $branch and .head_sha == $sha' <<<"$proof" >/dev/null || die 'latest run does not identify this promotion'
if ! jq -e '.workflow_runs[0].status == "completed"' <<<"$proof" >/dev/null; then
    jq -r '"promotion-proof: run \(.workflow_runs[0].id) is \(.workflow_runs[0].status)"' <<<"$proof" >&2
    exit 2
fi
identity="$(jq -er '.workflow_runs[0] | select(.conclusion == "success") | [.id,.run_attempt] | @tsv' <<<"$proof")" || die 'latest promotion run did not succeed'
IFS=$'\t' read -r run_id attempt <<<"$identity"
[[ "$run_id" =~ ^[0-9]+$ && "$attempt" =~ ^[1-9][0-9]*$ ]] || die 'missing promotion run identity/attempt'
gh api --paginate "repos/$repo/actions/runs/$run_id/attempts/$attempt/jobs?per_page=100" | jq -se '
    [.[].jobs[]] as $jobs |
    ["Gates passed", "Verify frozen promotion"] | all(.[]; . as $name |
      [$jobs[] | select(.name == $name)] |
      length == 1 and all(.[]; .status == "completed" and .conclusion == "success"))' >/dev/null || die 'promotion aggregate and frozen proof must each succeed exactly once in this run attempt'
get_proof | jq -e --arg sha "$head" --arg branch "$ref" --argjson id "$run_id" --argjson attempt "$attempt" '
    .workflow_runs[0] | .id == $id and .run_attempt == $attempt and .event == "pull_request" and .head_branch == $branch and .head_sha == $sha and .status == "completed" and .conclusion == "success"' >/dev/null || die 'promotion proof changed during verification'
printf '%s\t%s\n' "$run_id" "$attempt"
