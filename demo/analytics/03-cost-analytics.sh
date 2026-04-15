#!/bin/bash
# DEMO: COST TRACKING ANALYTICS
# Summary, breakdown by model/agent, and cost trends
#
# What this does:
#   1. Shows a cost summary
#   2. Breaks down costs by model
#   3. Breaks down costs by agent
#   4. Shows cost trends over 7 days
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "ANALYTICS: COST TRACKING" "Summary, breakdown by model/agent, trends"

subheader "STEP 1: Cost Summary"
run_cli_indented analytics costs summary

subheader "STEP 2: Breakdown by Model"
run_cli_indented analytics costs breakdown --by model

subheader "STEP 3: Breakdown by Agent"
run_cli_indented analytics costs breakdown --by agent

subheader "STEP 4: Cost Trends"
run_cli_indented analytics costs trends --since 7d

header "COST ANALYTICS DEMO COMPLETE"
