#!/bin/bash
# DEMO: SESSION ANALYTICS
# Session statistics, trends, and real-time monitoring
#
# What this does:
#   1. Shows session stats
#   2. Shows session trends over 7 days
#   3. Shows currently active sessions
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "ANALYTICS: SESSIONS" "Session statistics, trends, real-time"

subheader "STEP 1: Session Stats"
run_cli_indented analytics sessions stats

subheader "STEP 2: Session Trends"
run_cli_indented analytics sessions trends --since 7d

subheader "STEP 3: Active Sessions"
run_cli_indented analytics sessions live

header "SESSION ANALYTICS DEMO COMPLETE"
