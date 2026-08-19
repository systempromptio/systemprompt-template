#!/bin/bash
# SEED DEMO TENANTS — Populate the enterprise console with a demo customer
# portfolio. Run after 00-preflight.sh, only as part of a demo flow: nothing
# here belongs on a clean install, and no boot path ever runs it.
#
# A cross-customer view with one row proves nothing: the seat bars, the budget
# bands, and the margin column only mean anything next to each other. These
# three tenants are deliberately different shapes — one large enterprise well
# inside its cap, one standard customer approaching it, and one suspended
# customer whose inference cost has overrun what its licence pays.
#
# What this populates:
#   1. Plans (placeholders corrected by plans.yaml on next publish) and the
#      demo organizations: Northwind (enterprise), Contoso (standard),
#      Initech (standard, suspended)
#   2. Departments and members per tenant (fixed UUIDs in the de000000-…
#      range — obviously synthetic, stable across databases, and parseable by
#      every auth path that treats users.id as a UUID)
#   3. Thirty days of ai_requests traffic per member, shaped so the three
#      tenants land in three different budget bands
#   4. plugin_usage_events across the last 7 days, if the table is empty
#
# Idempotent: everything is keyed to demo- ids / fixed UUIDs and guarded by
# ON CONFLICT, so re-running adds nothing and never edits anyone else's rows.
#
# Cost: Free. Direct SQL, no inference.

set -e

source "$(cd "$(dirname "$0")" && pwd)/_common.sh"

header "SEED DEMO TENANTS" "Populating the enterprise console demo portfolio"

DB_URL="$(grep database_url "$PROJECT_DIR/.systemprompt/profiles/$PROFILE/secrets.json" 2>/dev/null \
  | head -1 \
  | sed 's/.*"database_url".*"\(postgres[^"]*\)".*/\1/')"
if [[ -z "$DB_URL" ]] || ! command -v psql >/dev/null 2>&1; then
  fail "psql or database_url unavailable — demo tenants need direct DB access"
  exit 1
fi

subheader "STEP 1: Organizations, departments, members"
psql "$DB_URL" -v ON_ERROR_STOP=1 -q -f "$(cd "$(dirname "$0")" && pwd)/fixtures/demo-tenants.sql" > /dev/null
pass "Demo tenants seeded (northwind, contoso, initech)"

subheader "STEP 2: Dashboard usage events"
psql "$DB_URL" -v ON_ERROR_STOP=1 -q -f "$(cd "$(dirname "$0")" && pwd)/fixtures/demo-usage-events.sql"
pass "Usage events seeded (skipped if plugin_usage_events already has data)"

pass "Demo tenant portfolio ready — open the enterprise console"
