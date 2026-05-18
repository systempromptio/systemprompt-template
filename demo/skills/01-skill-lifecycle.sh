#!/bin/bash
# DEMO: SKILL LIFECYCLE — LIST, DISK CONFIG
# Read-only skill management operations.
#
# What this does:
#   1. Lists all database-synced skills with their on-disk config paths
#   2. Walks the nested skill directories on disk
#   3. Shows one skill in full (config + markdown body)
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "SKILLS: LIFECYCLE" "List, disk configuration"

subheader "STEP 1: List Database-Synced Skills"
run_cli_head 30 core skills list

subheader "STEP 2: Nested skill directories on disk"
SKILLS_DIR="$PROJECT_DIR/services/skills"
echo "  \$ ls $SKILLS_DIR/*/config.yaml"
echo ""
for cfg in "$SKILLS_DIR"/*/config.yaml; do
  [[ -f "$cfg" ]] || continue
  echo "$cfg" | sed "s|$PROJECT_DIR/||" | sed 's/^/    /'
done
echo ""

subheader "STEP 3: Show use_dangerous_secret (full config + instructions)"
run_cli_head 40 core skills show use_dangerous_secret

header "SKILL LIFECYCLE DEMO COMPLETE" "Showed: list, nested config layout"
