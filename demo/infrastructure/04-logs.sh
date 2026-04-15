#!/bin/bash
# DEMO: LOG STREAMING AND TRACING
#
# What this demonstrates:
#   1. Viewing recent log entries with time filtering
#   2. Log summaries aggregated by level
#   3. Searching logs by keyword
#   4. Execution traces for debugging request flows
#   5. AI request logs for LLM call inspection
#   6. MCP tool execution logs
#
# CLI commands used:
#   - systemprompt infra logs view --since 1h --limit 10
#   - systemprompt infra logs summary --since 24h
#   - systemprompt infra logs search "extension" --since 24h
#   - systemprompt infra logs trace list --limit 5
#   - systemprompt infra logs request list --limit 5
#   - systemprompt infra logs tools list --limit 5
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "INFRASTRUCTURE: LOGS & TRACING" "Log viewing, search, traces, AI requests, tool executions"

subheader "STEP 1: Recent Logs"
run_cli_head 15 infra logs view --since 1h --limit 10

subheader "STEP 2: Log Summary"
run_cli_indented infra logs summary --since 24h

subheader "STEP 3: Search Logs"
run_cli_head 15 infra logs search "extension" --since 24h

subheader "STEP 4: Execution Traces"
run_cli_head 20 infra logs trace list --limit 5

subheader "STEP 5: AI Requests"
run_cli_head 20 infra logs request list --limit 5

subheader "STEP 6: MCP Tool Executions"
run_cli_head 20 infra logs tools list --limit 5

header "LOGS DEMO COMPLETE" "Showed: view, summary, search, trace, request, tools"
