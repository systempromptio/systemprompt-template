#!/bin/bash
# DEMO: RATE LIMITING — RATE LIMIT CONFIGURATION AND ENFORCEMENT
# Read-only rate limit and security config inspection.
#
# What this does:
#   1. Shows current rate limit configuration
#   2. Shows security configuration
#   3. Shows server configuration
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "GOVERNANCE: RATE LIMITING" "Rate limit configuration and enforcement"

subheader "STEP 1: Current Rate Limit Config"
run_cli_indented admin config rate-limits show

subheader "STEP 2: Rate Limit Tier Comparison"
run_cli_indented admin config rate-limits compare

subheader "STEP 3: Security Config"
run_cli_indented admin config security show

subheader "STEP 4: Server Config"
run_cli_indented admin config server show

header "RATE LIMITING DEMO COMPLETE"
