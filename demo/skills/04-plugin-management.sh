#!/bin/bash
# DEMO: PLUGIN MANAGEMENT — PLUGINS, HOOKS, EXTENSIONS
# Read-only plugin and extension inspection.
#
# What this does:
#   1. Lists database-synced plugins
#   2. Shows plugin YAMLs on disk
#   3. Lists hooks
#   4. Shows loaded extensions and their capabilities
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "SKILLS: PLUGINS & HOOKS" "Plugin inspection, hook management"

subheader "STEP 1: List Database-Synced Plugins"
run_cli_head 30 core plugins list

subheader "STEP 2: Plugin YAMLs on disk"
PLUGINS_DIR="$PROJECT_DIR/services/plugins"
if [[ -d "$PLUGINS_DIR" ]]; then
  echo "  \$ ls $PLUGINS_DIR/*.yaml"
  echo ""
  ls "$PLUGINS_DIR"/*.yaml 2>/dev/null | sed "s|$PROJECT_DIR/||" | sed 's/^/    /'
  echo ""
  FIRST_PLUGIN_YAML=$(ls "$PLUGINS_DIR"/*.yaml 2>/dev/null | head -1 || true)
  if [[ -n "$FIRST_PLUGIN_YAML" ]]; then
    echo "  First plugin YAML (truncated):"
    echo ""
    head -30 "$FIRST_PLUGIN_YAML" | sed 's/^/    /'
    echo ""
  fi
fi

subheader "STEP 3: List Hooks"
run_cli_head 20 core hooks list

subheader "STEP 4: Validate Hooks"
run_cli_indented core hooks validate

subheader "STEP 5: Extensions"
run_cli_head 20 plugins list

subheader "STEP 6: Extension Capabilities"
run_cli_indented plugins capabilities

header "PLUGIN MANAGEMENT DEMO COMPLETE"
