#!/bin/bash
# DEMO: AI REQUEST ANALYTICS
# Volume, latency, model usage, and request trends
#
# What this does:
#   1. Shows request stats
#   2. Lists recent requests from the last 24 hours (top 20 lines)
#   3. Shows model usage distribution
#   4. Shows request trends over 7 days
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "ANALYTICS: AI REQUESTS" "Volume, latency, model usage, trends"

subheader "STEP 1: Request Stats"
run_cli_indented analytics requests stats

subheader "STEP 2: Recent Requests"
run_cli_head 20 analytics requests list --since 24h

subheader "STEP 3: Model Usage"
run_cli_indented analytics requests models

subheader "STEP 4: Request Trends"
run_cli_indented analytics requests trends --since 7d

header "REQUEST ANALYTICS DEMO COMPLETE"
