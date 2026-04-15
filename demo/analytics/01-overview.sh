#!/bin/bash
# DEMO: ANALYTICS OVERVIEW
# Dashboard summary across all metrics
#
# What this does:
#   1. Shows a 24-hour analytics overview (default)
#   2. Shows a 7-day analytics overview
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "ANALYTICS: OVERVIEW" "Dashboard summary across all metrics"

subheader "STEP 1: 24-Hour Overview"
run_cli_indented analytics overview

subheader "STEP 2: 7-Day Overview"
run_cli_indented analytics overview --since 7d

header "OVERVIEW DEMO COMPLETE"
