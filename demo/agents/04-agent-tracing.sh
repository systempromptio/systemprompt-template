#!/bin/bash
# AGENTS: EXECUTION TRACING — Traces, artifacts, cost attribution
# Read-only demo showing traces from previous agent runs. No AI cost.
#
# Cost: Free (read-only CLI commands)
#
# Usage:
#   ./demo/agents/04-agent-tracing.sh

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "AGENTS: EXECUTION TRACING" "Traces, artifacts, cost attribution"

subheader "STEP 1: Recent Execution Traces"
run_cli_head 20 infra logs trace list --limit 5

subheader "STEP 2: Trace Detail"
info "Showing most recent trace..."
TRACE_ID=$("$CLI" infra logs trace list --limit 1 --profile "$PROFILE" 2>&1 | sed -n 's/.*"trace_id"[[:space:]]*:[[:space:]]*"\([0-9a-f-]*\)".*/\1/p' | head -1 || true)
if [[ -n "$TRACE_ID" ]]; then
  run_cli_head 40 infra logs trace show "$TRACE_ID" --all
else
  info "No traces yet — run SEED_AGENT_RUN=1 ./demo/01-seed-data.sh to populate one."
fi

subheader "STEP 3: Artifacts"
run_cli_head 20 core artifacts list

subheader "STEP 4: Cost Breakdown by Agent"
run_cli_indented analytics costs breakdown --by agent

header "AGENT TRACING DEMO COMPLETE"
