#!/bin/bash
# DEMO: SKILL LIFECYCLE — LIST, SHOW, STATUS
# Read-only skill management operations.
#
# What this does:
#   1. Lists all available skills
#   2. Shows details for a specific skill
#   3. Checks skill sync status
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "SKILLS: LIFECYCLE" "List, show, create, edit, sync, delete"

subheader "STEP 1: List Skills"
run_cli_head 30 core skills list

subheader "STEP 2: Show a Skill"
run_cli_head 30 core skills show general_assistance

subheader "STEP 3: Sync Status"
run_cli_indented core skills status

header "SKILL LIFECYCLE DEMO COMPLETE" "Showed: list, show, status"
