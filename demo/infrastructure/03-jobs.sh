#!/bin/bash
# DEMO: BACKGROUND JOBS
#
# What this demonstrates:
#   1. Listing available background jobs
#   2. Viewing job configuration and schedule details
#   3. Execution history for past job runs
#
# CLI commands used:
#   - systemprompt infra jobs list
#   - systemprompt infra jobs show <job_name>
#   - systemprompt infra jobs history
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "INFRASTRUCTURE: BACKGROUND JOBS" "Scheduling, monitoring, execution history"

subheader "STEP 1: List Available Jobs"
run_cli_indented infra jobs list

subheader "STEP 2: Job Details"
info "Showing details for the first available job..."
echo ""
# Get first job name
JOB_NAME=$("$CLI" infra jobs list --profile "$PROFILE" 2>&1 | sed -n 's/.*"name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -1 || echo "cleanup-sessions")
run_cli_indented infra jobs show "$JOB_NAME"

subheader "STEP 3: Execution History"
run_cli_head 20 infra jobs history

header "JOBS DEMO COMPLETE" "Showed: list, show, history"
