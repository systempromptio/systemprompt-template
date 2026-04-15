#!/bin/bash
# DEMO: TOOL USAGE ANALYTICS — Stats, listings, trends
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "ANALYTICS: TOOL USAGE" "Tool statistics, listings, trends"

subheader "STEP 1: Tool Stats"
run_cli_indented analytics tools stats

subheader "STEP 2: Tool List with Metrics"
run_cli_head 20 analytics tools list

subheader "STEP 3: Tool Trends"
run_cli_indented analytics tools trends --since 7d

header "TOOL ANALYTICS DEMO COMPLETE"
