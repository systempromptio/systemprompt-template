#!/bin/bash
# DEMO: PLUGIN MANAGEMENT — PLUGINS, HOOKS, EXTENSIONS
# Read-only plugin and extension inspection.
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "SKILLS: PLUGINS & HOOKS" "Plugin inspection, hook management"

subheader "STEP 1: List Database-Synced Plugins"
run_cli_head 30 core plugins list

subheader "STEP 2: Marketplace catalogues on disk"
# Content reaches the client via the marketplace -> synthetic
# `systemprompt-managed` plugin path, not via standalone plugin bundles.
# The marketplace config is the authoritative catalogue.
MARKETPLACES_DIR="$PROJECT_DIR/services/marketplaces"
echo "  \$ ls $MARKETPLACES_DIR/*/config.yaml"
echo ""
for cfg in "$MARKETPLACES_DIR"/*/config.yaml; do
  [[ -f "$cfg" ]] || continue
  echo "$cfg" | sed "s|$PROJECT_DIR/||" | sed 's/^/    /'
done
echo ""

subheader "STEP 3: Show enterprise-demo marketplace catalogue"
echo "  \$ cat services/marketplaces/enterprise-demo/config.yaml"
echo ""
sed 's/^/    /' "$MARKETPLACES_DIR/enterprise-demo/config.yaml" | head -40

subheader "STEP 4: List Hooks"
run_cli_head 20 core hooks list

subheader "STEP 5: Validate Hooks"
run_cli_indented core hooks validate
# Each row carries a valid flag; assert at least one hook set and none invalid.
HOOKS_JSON=$(cli_json core hooks validate)
assert_min "$(printf '%s' "$HOOKS_JSON" | jq '.items | length')" \
  1 "plugin hook set(s) present"
assert_eq "$(printf '%s' "$HOOKS_JSON" | jq '[.items[]|select(.valid!="true" and .valid!=true)]|length')" \
  "0" "all plugin hook sets valid"

subheader "STEP 6: Extensions"
run_cli_head 20 plugins list

subheader "STEP 7: Extension Capabilities"
run_cli_indented plugins capabilities

header "PLUGIN MANAGEMENT DEMO COMPLETE"
