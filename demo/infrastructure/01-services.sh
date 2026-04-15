#!/bin/bash
# DEMO: SERVICE LIFECYCLE MANAGEMENT
#
# What this demonstrates:
#   1. Service status overview — running agents and MCP servers
#   2. Detailed status with process info, ports, and health
#   3. Lifecycle commands (start/stop/restart) and cleanup
#
# CLI commands used:
#   - systemprompt infra services status
#   - systemprompt infra services status --detailed
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "INFRASTRUCTURE: SERVICE MANAGEMENT" "Service lifecycle, health checks, process status"

subheader "STEP 1: Service Status"
run_cli_indented infra services status

subheader "STEP 2: Detailed Status"
run_cli_head 30 infra services status --detailed

subheader "STEP 3: Summary"
info "Services manage 3 agents + 2 MCP servers."
info "Use 'infra services start/stop/restart' for lifecycle."
info "Use 'infra services cleanup --yes' to fix orphaned processes."

echo ""
header "SERVICES DEMO COMPLETE" "Showed: status, detailed status, lifecycle overview"
