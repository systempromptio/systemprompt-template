#!/bin/bash
# DEMO: USER MANAGEMENT — LISTING, COUNTS, STATS, SEARCH
# Read-only user management operations.
#
# What this does:
#   1. Lists all users
#   2. Shows user count
#   3. Displays user statistics
#   4. Searches users by keyword
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "USERS: MANAGEMENT" "Listing, counts, stats, search"

subheader "STEP 1: List Users"
run_cli_head 20 admin users list

subheader "STEP 2: User Count"
run_cli_indented admin users count

subheader "STEP 3: User Statistics"
run_cli_indented admin users stats

subheader "STEP 4: Search Users"
run_cli_head 20 admin users search "admin"

header "USER MANAGEMENT DEMO COMPLETE"
