#!/bin/bash
# DEMO: CONTEXT MANAGEMENT — Create, list, show, edit, delete
# Contexts are conversation containers for agent sessions.
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "SKILLS: CONTEXT MANAGEMENT" "Create, list, show, edit, delete conversation contexts"

subheader "STEP 1: List Existing Contexts"
run_cli_head 20 core contexts list

subheader "STEP 2: Create a Demo Context"
cmd "systemprompt core contexts create --name \"Demo Context $(date +%H:%M:%S)\""
CONTEXT_OUTPUT=$("$CLI" core contexts create --name "Demo Context $(date +%H:%M:%S)" --profile "$PROFILE" 2>&1)
echo "$CONTEXT_OUTPUT" | sed 's/^/  /'
echo ""

CONTEXT_ID=$(echo "$CONTEXT_OUTPUT" | sed -n 's/.*"id":[[:space:]]*"\([^"]*\)".*/\1/p' | head -1 || true)
if [[ -z "$CONTEXT_ID" ]]; then
  CONTEXT_ID=$(echo "$CONTEXT_OUTPUT" | grep -oE '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}' | head -1 || true)
fi

if [[ -n "$CONTEXT_ID" ]]; then
  subheader "STEP 3: Show Context Details"
  run_cli_indented core contexts show "$CONTEXT_ID"

  subheader "STEP 4: Rename Context"
  run_cli_indented core contexts edit "$CONTEXT_ID" --name "Renamed Demo Context"

  subheader "STEP 5: Verify Rename"
  run_cli_indented core contexts show "$CONTEXT_ID"

  subheader "STEP 6: Delete Context (cleanup)"
  run_cli_indented core contexts delete "$CONTEXT_ID"
else
  info "Could not extract context ID — skipping show/edit/delete steps."
fi

subheader "STEP 7: List Contexts (after cleanup)"
run_cli_head 20 core contexts list

header "CONTEXT MANAGEMENT DEMO COMPLETE"
