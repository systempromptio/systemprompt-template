#!/bin/bash
# DEMO: AGENT PERFORMANCE ANALYTICS
# Stats, rankings, trends, and deep-dives into agent performance
#
# What this does:
#   1. Shows aggregate agent stats
#   2. Lists agents with metrics (top 20 lines)
#   3. Shows agent trends over 7 days
#   4. Deep-dives into developer_agent details (top 30 lines)
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "ANALYTICS: AGENT PERFORMANCE" "Stats, rankings, trends, deep-dives"

subheader "STEP 1: Aggregate Stats"
run_cli_indented analytics agents stats

subheader "STEP 2: Agent List with Metrics"
run_cli_head 20 analytics agents list

subheader "STEP 3: Agent Trends"
run_cli_indented analytics agents trends --since 7d

subheader "STEP 4: Deep Dive"
info "Showing developer_agent details..."
run_cli_head 30 analytics agents show developer_agent

header "AGENT ANALYTICS DEMO COMPLETE"
