#!/bin/bash
# app-scheduler — job scheduling
set -e
source "$(dirname "$0")/../_colors.sh"

header "app-scheduler" "Cron-like jobs driven from services/scheduler/config.yaml"
pause 0.8

type_cmd "systemprompt infra jobs list"
pause 0.2
"$CLI" infra jobs list --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 0.6

type_cmd "systemprompt infra jobs show blog_content_ingestion"
pause 0.2
"$CLI" infra jobs show blog_content_ingestion --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""
pass "declarative job schedules, zero sidecar cron"
pause 1.2
