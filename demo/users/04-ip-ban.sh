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

TEST_IP="192.168.99.99"
# Count how many ban rows match a given IP, read from structured --json output.
ban_count_for() {
  cli_json admin users ban list | jq --arg ip "$1" '[.items[]?|select(.ip_address==$ip)]|length'
}

header "USERS: IP BAN MANAGEMENT" "View and manage IP bans — add/verify/remove round-trip"

subheader "STEP 1: Current Ban List"
run_cli_indented admin users ban list

subheader "STEP 2: Add Test Ban"
info "Adding temporary ban for test IP $TEST_IP..."
cmd "systemprompt admin users ban add $TEST_IP --reason \"demo test\""
"$CLI" admin users ban add "$TEST_IP" --reason "demo test" --profile "$PROFILE" 2>&1 | sed 's/^/  /' || true

subheader "STEP 3: Verify Ban Was Added"
run_cli_indented admin users ban list
assert_eq "$(ban_count_for "$TEST_IP")" "1" "ban for $TEST_IP is present after add"

subheader "STEP 4: Remove Test Ban"
info "Cleaning up test ban..."
cmd "systemprompt admin users ban remove $TEST_IP"
"$CLI" admin users ban remove "$TEST_IP" --yes --profile "$PROFILE" 2>&1 | sed 's/^/  /' || true

subheader "STEP 5: Confirm Removal"
run_cli_indented admin users ban list
assert_eq "$(ban_count_for "$TEST_IP")" "0" "ban for $TEST_IP is gone after remove"

header "IP BAN DEMO COMPLETE"
