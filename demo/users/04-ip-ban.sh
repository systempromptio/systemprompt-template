#!/bin/bash
# DEMO: IP BAN MANAGEMENT — VIEW AND MANAGE IP BANS
# Demonstrates ban add/remove with cleanup.
#
# What this does:
#   1. Shows current ban list
#   2. Adds a temporary test ban
#   3. Verifies the ban was added
#   4. Removes the test ban
#   5. Confirms removal
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "USERS: IP BAN MANAGEMENT" "View and manage IP bans"

subheader "STEP 1: Current Ban List"
run_cli_indented admin users ban list

subheader "STEP 2: Add Test Ban"
info "Adding temporary ban for test IP 192.168.99.99..."
cmd "systemprompt admin users ban add 192.168.99.99 --reason \"demo test\""
"$CLI" admin users ban add 192.168.99.99 --reason "demo test" --profile "$PROFILE" 2>&1 | sed 's/^/  /' || true

subheader "STEP 3: Verify Ban"
run_cli_indented admin users ban list

subheader "STEP 4: Remove Test Ban"
info "Cleaning up test ban..."
cmd "systemprompt admin users ban remove 192.168.99.99"
"$CLI" admin users ban remove 192.168.99.99 --yes --profile "$PROFILE" 2>&1 | sed 's/^/  /' || true

subheader "STEP 5: Confirm Removal"
run_cli_indented admin users ban list

header "IP BAN DEMO COMPLETE"
