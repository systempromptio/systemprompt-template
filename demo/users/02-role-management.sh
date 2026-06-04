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
# Pull the first user id from structured output — a text-parse of the box table
# yields empty and falsely reports "No users in DB" even with users present.
USER_ID=$(json_first '.items[0].id' admin users list)
assert_nonempty "$USER_ID" "at least one user exists to inspect"
run_cli_head 30 admin users show "$USER_ID"

subheader "STEP 3: User Role Statistics"
run_cli_indented admin users stats

header "ROLE MANAGEMENT DEMO COMPLETE"
