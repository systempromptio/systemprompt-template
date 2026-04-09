#!/bin/bash
# DEMO: CONTENT & TRAFFIC ANALYTICS
# Content engagement, top pages, traffic sources, geographic distribution, devices
#
# What this does:
#   1. Shows content stats
#   2. Lists top content (top 20 lines)
#   3. Shows content trends over 7 days
#   4. Shows traffic sources
#   5. Shows geographic distribution
#   6. Shows device breakdown
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "ANALYTICS: CONTENT & TRAFFIC" "Content engagement, top pages, traffic sources, geo, devices"

subheader "STEP 1: Content Stats"
run_cli_indented analytics content stats

subheader "STEP 2: Top Content"
run_cli_head 20 analytics content top

subheader "STEP 3: Content Trends"
run_cli_indented analytics content trends --since 7d

subheader "STEP 4: Traffic Sources"
run_cli_indented analytics traffic sources

subheader "STEP 5: Geographic Distribution"
run_cli_indented analytics traffic geo

subheader "STEP 6: Device Breakdown"
run_cli_indented analytics traffic devices

header "CONTENT & TRAFFIC DEMO COMPLETE"
