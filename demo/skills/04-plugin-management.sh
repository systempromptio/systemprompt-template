#!/bin/bash
# DEMO: PLUGIN MANAGEMENT — PLUGINS, HOOKS, EXTENSIONS
# Read-only plugin and hook inspection.
#
# What this does:
#   1. Lists core plugins
#   2. Shows plugin details
#   3. Validates a plugin
#   4. Lists and validates hooks
#   5. Shows extensions and capabilities
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "SKILLS: PLUGINS & HOOKS" "Plugin inspection, validation, hook management"

subheader "STEP 1: List Plugins (Core)"
run_cli_head 30 core plugins list

subheader "STEP 2: Show Plugin Details"
run_cli_head 30 core plugins show enterprise-demo

subheader "STEP 3: Validate Plugin"
run_cli_indented core plugins validate enterprise-demo

subheader "STEP 4: List Hooks"
run_cli_head 20 core hooks list

subheader "STEP 5: Validate Hooks"
run_cli_indented core hooks validate

subheader "STEP 6: Extensions"
run_cli_head 20 plugins list

subheader "STEP 7: Extension Capabilities"
run_cli_indented plugins capabilities

header "PLUGIN MANAGEMENT DEMO COMPLETE"
