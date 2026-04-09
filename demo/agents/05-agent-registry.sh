#!/bin/bash
# AGENTS: REGISTRY & LOGS — A2A discovery, running agents, process logs
# Shows the A2A agent registry and per-agent process logs.
#
# Cost: Free (read-only CLI commands)
#
# Usage:
#   ./demo/agents/05-agent-registry.sh

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "AGENTS: REGISTRY & LOGS" "A2A discovery, running agents, process logs"

subheader "STEP 1: Agent Registry (A2A Gateway)"
run_cli_head 30 admin agents registry

subheader "STEP 2: Agent Logs — Developer Agent"
run_cli_head 20 admin agents logs developer_agent

subheader "STEP 3: Agent Logs — Associate Agent"
run_cli_head 20 admin agents logs associate_agent

header "AGENT REGISTRY DEMO COMPLETE"
