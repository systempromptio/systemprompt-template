#!/bin/bash
# DEMO: ROLE MANAGEMENT — USER DETAILS, ROLE INFORMATION
# Read-only role and permission inspection.
#
# What this does:
#   1. Lists users with their roles
#   2. Shows details for the first user found
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "USERS: ROLES & PERMISSIONS" "View user details, role information"

subheader "STEP 1: List Users with Roles"
run_cli_head 20 admin users list

subheader "STEP 2: User Details"
info "Showing first user's details..."
USER_ID=$("$CLI" admin users list --profile "$PROFILE" 2>&1 | sed -n 's/.*"id":[[:space:]]*"\([^"]*\)".*/\1/p' | head -1 || true)
if [[ -n "$USER_ID" ]]; then
  run_cli_head 30 admin users show "$USER_ID"
else
  info "No users in DB yet — authenticate once via the web UI to seed one."
fi

subheader "STEP 3: User Role Statistics"
run_cli_indented admin users stats

header "ROLE MANAGEMENT DEMO COMPLETE"
