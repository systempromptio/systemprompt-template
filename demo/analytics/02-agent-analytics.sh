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
info "Showing developer_agent details (requires prior agent activity)..."
cmd "systemprompt analytics agents show developer_agent"
"$CLI" analytics agents show developer_agent --profile "$PROFILE" 2>&1 | head -30 | sed 's/^/  /' || info "No agent data available yet. Run agents/03-agent-messaging.sh first."
echo ""

header "AGENT ANALYTICS DEMO COMPLETE"
