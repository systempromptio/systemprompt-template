#!/bin/bash
# DEMO: CONFIGURATION OVERVIEW — Full platform configuration
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "INFRASTRUCTURE: CONFIGURATION" "Full platform configuration overview"

subheader "STEP 1: Configuration Overview"
run_cli_indented admin config show

subheader "STEP 2: Configuration Files"
run_cli_head 30 admin config list

subheader "STEP 3: Validate Configuration"
run_cli_indented admin config validate

subheader "STEP 4: Runtime Configuration"
run_cli_indented admin config runtime show

subheader "STEP 5: Paths Configuration"
run_cli_indented admin config paths show

subheader "STEP 6: AI Provider Configuration"
run_cli_indented admin config provider list

header "CONFIGURATION DEMO COMPLETE"
