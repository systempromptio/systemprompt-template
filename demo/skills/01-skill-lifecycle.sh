#!/bin/bash
# DEMO: SKILL LIFECYCLE — LIST, STATUS, DISK CONFIG
# Read-only skill management operations.
#
# What this does:
#   1. Lists all database-synced skills
#   2. Shows disk/db sync status
#   3. Lists skill YAMLs under services/skills/
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "SKILLS: LIFECYCLE" "List, status, disk configuration"

subheader "STEP 1: List Database-Synced Skills"
run_cli_head 30 core skills list

subheader "STEP 2: Disk/DB Sync Status"
run_cli_indented core skills status

subheader "STEP 3: Skill YAMLs on disk"
SKILLS_DIR="$PROJECT_DIR/services/skills"
if [[ -d "$SKILLS_DIR" ]]; then
  echo "  \$ ls $SKILLS_DIR/*.yaml"
  echo ""
  ls "$SKILLS_DIR"/*.yaml 2>/dev/null | sed "s|$PROJECT_DIR/||" | sed 's/^/    /'
  echo ""
  FIRST_SKILL_YAML=$(ls "$SKILLS_DIR"/*.yaml 2>/dev/null | head -1 || true)
  if [[ -n "$FIRST_SKILL_YAML" ]]; then
    echo "  First skill YAML:"
    echo ""
    head -20 "$FIRST_SKILL_YAML" | sed 's/^/    /'
    echo ""
  fi
else
  info "services/skills/ not found"
fi

header "SKILL LIFECYCLE DEMO COMPLETE" "Showed: list, status, disk config"
