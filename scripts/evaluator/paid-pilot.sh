#!/usr/bin/env bash
# Executes the fixed eight-case baseline/candidate matrix under one shared
# $5 account. It records budget-blocked without shrinking the matrix.
set -euo pipefail

: "${BASE_URL:?BASE_URL is required}"
: "${ORIGIN:?ORIGIN is required}"
: "${PAT_FILE:?PAT_FILE is required}"
: "${SPEC_TEMPLATE:?SPEC_TEMPLATE is required}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"
mkdir -p "$EVIDENCE_DIR"
token="$(<"$PAT_FILE")"

api() {
    method="$1" path="$2" body="${3:-}"
    args=(-fsS -X "$method" -H "Authorization: Bearer $token" -H "Origin: $ORIGIN")
    if [ -n "$body" ]; then args+=(-H 'Content-Type: application/json' --data-binary "@$body"); fi
    curl "${args[@]}" "$BASE_URL/api/public/admin$path"
}

api POST /evals/suites/super-admin | tee "$EVIDENCE_DIR/suite.json" >/dev/null
pilot_ids='["daily-pagination","daily-repeat","critical-recorded-derived","critical-demo-only","usage-currency","usage-authorized-write","cli-help","cli-write-timeout"]'
case_revisions="$(jq -c --argjson wanted "$pilot_ids" '[.cases[] | select(.authored_id as $id | $wanted | index($id)) | .revision_id]' "$EVIDENCE_DIR/suite.json")"
test "$(jq length <<<"$case_revisions")" = 8
test "$(jq -r --argjson wanted "$pilot_ids" '[.cases[] | select(.authored_id as $id | $wanted | index($id)) | .skill_id] | group_by(.) | map(length == 2) | all' "$EVIDENCE_DIR/suite.json")" = true

jq --argjson cases "$case_revisions" \
   --arg dataset "$(jq -er '.dataset_revision_id' "$EVIDENCE_DIR/suite.json")" \
   --arg rubric "$(jq -er '.rubric_revision_id' "$EVIDENCE_DIR/suite.json")" \
   --arg dataset_digest "$(jq -er '.dataset_digest' "$EVIDENCE_DIR/suite.json")" \
   --arg rubric_digest "$(jq -er '.rubric_digest' "$EVIDENCE_DIR/suite.json")" \
   '.cases=$cases | .dataset=$dataset | .rubric=$rubric | .repetitions=1 | .frozen.dataset_digest=$dataset_digest | .frozen.rubric_digest=$rubric_digest' \
   "$SPEC_TEMPLATE" > "$EVIDENCE_DIR/spec.json"

if grep -q 'REPLACE_' "$EVIDENCE_DIR/spec.json"; then
    echo "pilot spec still contains REPLACE_ placeholders" >&2
    exit 2
fi
test "$(jq '.variants | length' "$EVIDENCE_DIR/spec.json")" = 2
test "$(jq -r '.variants[0].skill_bundle_digest != .variants[1].skill_bundle_digest and .variants[0].client == "claude-code" and .variants[1].client == "claude-code"' "$EVIDENCE_DIR/spec.json")" = true
executions=16
attempts="$(jq -er '.frozen.cost_envelope.maximum_attempts_per_execution' "$EVIDENCE_DIR/spec.json")"
per_attempt="$(jq '[.frozen.cost_envelope.generation_microdollars_per_attempt,.frozen.cost_envelope.judging_microdollars_per_attempt,.frozen.cost_envelope.tool_microdollars_per_attempt] | add' "$EVIDENCE_DIR/spec.json")"
suggestions="$(jq '.frozen.cost_envelope.suggestion_calls * .frozen.cost_envelope.suggestion_microdollars_per_call' "$EVIDENCE_DIR/spec.json")"
auxiliary="$(jq '.frozen.cost_envelope.auxiliary_calls * .frozen.cost_envelope.auxiliary_microdollars_per_call' "$EVIDENCE_DIR/spec.json")"
maximum=$((executions * attempts * per_attempt + suggestions + auxiliary))
test "$maximum" -gt 0
jq --argjson maximum "$maximum" '.budget_microdollars=$maximum' "$EVIDENCE_DIR/spec.json" > "$EVIDENCE_DIR/spec.final.json"
mv "$EVIDENCE_DIR/spec.final.json" "$EVIDENCE_DIR/spec.json"
jq -n --argjson executions "$executions" --argjson maximum "$maximum" --argjson cap 5000000 \
    '{executions:$executions,maximum_microdollars:$maximum,account_cap_microdollars:$cap,includes_generation:true,includes_judging:true,includes_retries:true,includes_suggestions:true,includes_auxiliary:true}' > "$EVIDENCE_DIR/cost-preflight.json"
if [ "$maximum" -gt 5000000 ]; then
    jq -n --argjson maximum "$maximum" '{status:"budget-blocked",reason:"complete fixed matrix exceeds $5",maximum_microdollars:$maximum}' | tee "$EVIDENCE_DIR/result.json"
    exit 3
fi

jq -n '{idempotency_key:"super-admin-paid-pilot-account-v1",cap_microdollars:5000000}' > /tmp/pilot-budget.json
api POST /evals/budgets /tmp/pilot-budget.json | tee "$EVIDENCE_DIR/budget.json" >/dev/null
budget_id="$(jq -er '.id' "$EVIDENCE_DIR/budget.json")"
jq -n --arg key super-admin-paid-pilot-v1 --arg budget "$budget_id" --slurpfile spec "$EVIDENCE_DIR/spec.json" \
    '{idempotency_key:$key,budget_id:$budget,spec:$spec[0]}' > /tmp/pilot-launch.json
api POST /evals/experiments/preflight /tmp/pilot-launch.json | tee "$EVIDENCE_DIR/server-preflight.json" >/dev/null
test "$(jq -er '.execution_count' "$EVIDENCE_DIR/server-preflight.json")" = 16
test "$(jq -er '.maximum_cost_microdollars' "$EVIDENCE_DIR/server-preflight.json")" = "$maximum"
test "$(jq -er '.affordable' "$EVIDENCE_DIR/server-preflight.json")" = true
api POST /evals/experiments /tmp/pilot-launch.json | tee "$EVIDENCE_DIR/launch.json" >/dev/null
jq -n --arg id "$(jq -er '.id' "$EVIDENCE_DIR/launch.json")" --arg budget "$budget_id" --argjson maximum "$maximum" \
    '{status:"launched",experiment_id:$id,budget_id:$budget,maximum_microdollars:$maximum,matrix:"8 cases x 2 variants x 1 repetition"}' | tee "$EVIDENCE_DIR/result.json"
