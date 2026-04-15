#!/bin/bash
# DEMO: HOOKS — Hook listing and validation
# Shows all configured hooks across plugins.
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "GOVERNANCE: HOOKS" "Hook configuration and validation"

subheader "STEP 1: List All Hooks"
run_cli_head 40 core hooks list

subheader "STEP 2: Validate Hooks"
run_cli_indented core hooks validate

header "HOOKS DEMO COMPLETE"
