#!/bin/bash
# SEED USAGE EVENTS — Populate plugin_usage_events with demo dashboard data.
# Run after 00-preflight.sh, only as part of a demo flow: nothing here belongs
# on a clean install, and no boot path ever runs it. Only inserts when the
# table is empty.

set -e

source "$(cd "$(dirname "$0")" && pwd)/_common.sh"

header "SEED USAGE EVENTS" "Populating dashboard demo usage events"

DB_URL="$(grep database_url "$PROJECT_DIR/.systemprompt/profiles/$PROFILE/secrets.json" 2>/dev/null \
  | head -1 \
  | sed 's/.*"database_url".*"\(postgres[^"]*\)".*/\1/')"
if [[ -z "$DB_URL" ]] || ! command -v psql >/dev/null 2>&1; then
  fail "psql or database_url unavailable — usage events need direct DB access"
  exit 1
fi

psql "$DB_URL" -v ON_ERROR_STOP=1 -q -f "$(cd "$(dirname "$0")" && pwd)/fixtures/demo-usage-events.sql"
pass "Usage events seeded (skipped if plugin_usage_events already has data)"
