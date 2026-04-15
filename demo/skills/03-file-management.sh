#!/bin/bash
# DEMO: FILE MANAGEMENT — LISTING, CONFIG, STATISTICS
# Read-only file management operations.
#
# What this does:
#   1. Lists managed files
#   2. Shows upload configuration
#   3. Displays file statistics
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "SKILLS: FILE MANAGEMENT" "File listing, config, statistics"

subheader "STEP 1: List Files"
run_cli_head 20 core files list

subheader "STEP 2: Upload Config"
run_cli_indented core files config

subheader "STEP 3: File Statistics"
run_cli_indented core files stats

header "FILE MANAGEMENT DEMO COMPLETE"
