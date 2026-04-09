#!/bin/bash
# DEMO: CONTENT MANAGEMENT — LISTING, SEARCH, STATUS
# Read-only content management operations.
#
# What this does:
#   1. Lists available content
#   2. Searches content by keyword
#   3. Shows popular content
#   4. Checks content status for a source
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "SKILLS: CONTENT MANAGEMENT" "Content listing, search, status"

subheader "STEP 1: List Content"
run_cli_head 30 core content list

subheader "STEP 2: Search Content"
run_cli_head 20 core content search "governance"

subheader "STEP 3: Popular Content"
run_cli_head 20 core content popular

subheader "STEP 4: Content Status"
run_cli_indented core content status --source documentation

header "CONTENT MANAGEMENT DEMO COMPLETE"
