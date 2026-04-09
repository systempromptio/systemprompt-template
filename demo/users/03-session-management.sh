#!/bin/bash
# DEMO: SESSION MANAGEMENT — SESSION LISTING, PROFILE SWITCHING
# Read-only session inspection.
#
# What this does:
#   1. Shows current session details
#   2. Lists available profiles
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "USERS: SESSION MANAGEMENT" "Session listing, profile switching"

subheader "STEP 1: Current Session"
run_cli_indented admin session show

subheader "STEP 2: Available Profiles"
run_cli_indented admin session list

header "SESSION MANAGEMENT DEMO COMPLETE"
