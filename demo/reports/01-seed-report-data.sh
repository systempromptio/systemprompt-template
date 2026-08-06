#!/bin/bash
# SEED REPORT DATA — realistic history behind the two month-end reports.
#
# Fills the existing Astound Digital and systemprompt organizations with
# departments, ~50 users, and six complete calendar months of ai_requests
# spread across models, so /admin/reports/internal and
# /admin/reports/customer render something a human can judge.
#
# Every row this writes carries an `rptseed-` id, and nothing pre-existing is
# updated or deleted. That is what makes 02-unseed-report-data.sh an exact
# restore rather than a best effort — see the header there.
#
# Re-running is safe: the unseed block runs first, so a second seed replaces
# its own rows rather than doubling them.
#
# Cost: free. No AI calls, no network.

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "SEED REPORT DATA" "Two enterprises, 6 months of usage"

DB_URL="$(grep database_url "$PROJECT_DIR/.systemprompt/profiles/$PROFILE/secrets.json" 2>/dev/null \
  | head -1 \
  | sed 's/.*"database_url".*"\(postgres[^"]*\)".*/\1/')"

if [[ -z "$DB_URL" ]] || ! command -v psql >/dev/null 2>&1; then
  fail "psql or database_url unavailable — cannot seed report data"
  exit 1
fi

# The organizations are authored in services/access-control/plans.yaml and
# bootstrapped at startup, so this script attaches to them rather than creating
# its own. Seeding into a customer that does not exist would produce a report
# about nobody.
MISSING=$(psql "$DB_URL" -t -A -c \
  "SELECT string_agg(want.slug, ', ')
     FROM (VALUES ('astound-digital'), ('systemprompt')) AS want(slug)
     LEFT JOIN organizations o ON o.slug = want.slug
    WHERE o.id IS NULL;")
if [[ -n "${MISSING// /}" ]]; then
  fail "Missing organization(s): $MISSING"
  info "Run the server once so plans.yaml bootstraps them, then re-run this script."
  exit 1
fi

subheader "STEP 1: Clear any previous seed"
bash "$(dirname "$0")/02-unseed-report-data.sh" --quiet
echo ""

subheader "STEP 2: Departments and users on the existing organizations"
psql "$DB_URL" -v ON_ERROR_STOP=1 -q <<'SQL' > /dev/null
BEGIN;

-- Departments are a display nicety on the enterprise page; the reports group
-- by user_profile_ext.department, which is free text. Skip any name already
-- taken, because the unique constraint is (name) before migration 022 and
-- (org_id, name) after it, and this script must work on both.
INSERT INTO departments (id, name, description, org_id)
SELECT 'rptseed-dept-' || lower(replace(d.name, ' ', '-')) || '-' || d.slug,
       d.name, 'Seeded for the month-end report demo.', o.id
FROM (
  VALUES
    ('Engineering', 'astound-digital'),
    ('Product',     'astound-digital'),
    ('Design',      'astound-digital'),
    ('Sales',       'astound-digital'),
    ('Support',     'astound-digital'),
    ('Growth',      'systemprompt')
) AS d(name, slug)
JOIN organizations o ON o.slug = d.slug
WHERE NOT EXISTS (SELECT 1 FROM departments x WHERE x.name = d.name);

-- 40 Astound users across five departments, 12 systemprompt users across two.
-- The department a user sits in is derived from their index, so the split is
-- stable across re-seeds and the by-department table is reproducible.
WITH spec AS (
  SELECT 'astound-digital' AS slug, 'astound' AS tag, 40 AS headcount,
         ARRAY['Engineering','Engineering','Engineering','Product','Design','Sales','Support']::TEXT[] AS depts,
         'astound-seed.example' AS domain
  UNION ALL
  SELECT 'systemprompt', 'sp', 12,
         ARRAY['Engineering','Growth']::TEXT[],
         'systemprompt-seed.example'
),
people AS (
  SELECT
    'rptseed-user-' || s.tag || '-' || lpad(i::TEXT, 3, '0') AS id,
    o.id AS org_id,
    s.depts[1 + (i % array_length(s.depts, 1))] AS department,
    'rptseed.' || s.tag || '.' || lpad(i::TEXT, 3, '0') || '@' || s.domain AS email,
    initcap(s.tag) || ' Seed ' || lpad(i::TEXT, 3, '0') AS display_name,
    i
  FROM spec s
  JOIN organizations o ON o.slug = s.slug
  CROSS JOIN generate_series(1, s.headcount) AS i
),
ins_users AS (
  INSERT INTO users (id, name, email, full_name, display_name, status,
                     email_verified, roles)
  SELECT id, id, email, display_name, display_name, 'active', TRUE,
         ARRAY['user']::TEXT[]
  FROM people
  RETURNING id
),
ins_members AS (
  INSERT INTO organization_members (user_id, org_id, org_role)
  SELECT id, org_id,
         CASE WHEN i = 1 THEN 'owner' WHEN i = 2 THEN 'admin' ELSE 'member' END
  FROM people
  RETURNING user_id
)
INSERT INTO user_profile_ext (user_id, department)
SELECT id, department FROM people;

COMMIT;
SQL
pass "52 users across Astound Digital and systemprompt"
echo ""

subheader "STEP 3: Six complete months of inference requests"
# Every timestamp is placed strictly inside a completed calendar month. The
# reports use half-open month bounds, so a row on a boundary would land in the
# wrong month and the totals would not reconcile against the request log.
psql "$DB_URL" -v ON_ERROR_STOP=1 -q <<'SQL' > /dev/null
WITH catalog AS (
  -- Rates are microdollars per million tokens.
  SELECT * FROM (VALUES
    (1, 'anthropic', 'claude-opus-4',   15000000, 75000000),
    (2, 'anthropic', 'claude-sonnet-4',  3000000, 15000000),
    (3, 'anthropic', 'claude-haiku-4',    800000,  4000000),
    (4, 'openai',    'gpt-4o',           2500000, 10000000)
  ) AS t(idx, provider, model, in_rate, out_rate)
),
members AS (
  SELECT m.user_id, o.slug,
         row_number() OVER (PARTITION BY m.org_id ORDER BY m.user_id) AS seat
  FROM organization_members m
  JOIN organizations o ON o.id = m.org_id
  WHERE m.user_id LIKE 'rptseed-%'
),
calls AS (
  SELECT
    mb.user_id,
    -- Weighted toward the cheap model, so it carries the call volume while the
    -- expensive one carries the spend. That is the shape a real bill has, and
    -- a flat split would make the by-model table say nothing.
    (ARRAY[1,2,2,3,3,3,3,4,2,3])[1 + ((mb.seat * 7 + n * 3 + back) % 10)] AS model_idx,
    DATE_TRUNC('month', NOW()) - (back || ' months')::INTERVAL
      + (random() * INTERVAL '27 days') AS ts
  FROM members mb
  CROSS JOIN generate_series(1, 6) AS back
  -- Volume grows month over month (back counts backwards), so the trend chart
  -- slopes instead of sitting flat.
  CROSS JOIN LATERAL generate_series(
    1,
    CASE WHEN mb.slug = 'astound-digital'
         THEN 34 + (6 - back) * 4 + (mb.seat % 7)::INT
         ELSE 18 + (6 - back) * 2 + (mb.seat % 5)::INT
    END
  ) AS n
),
priced AS (
  SELECT
    c.user_id,
    c.ts,
    cat.provider,
    cat.model,
    cat.in_rate,
    cat.out_rate,
    (400 + (random() * 12000))::INT AS input_tokens,
    (120 + (random() * 3000))::INT AS output_tokens,
    (random() * 4000)::INT AS cache_read_tokens,
    row_number() OVER () AS seq
  FROM calls c
  JOIN catalog cat ON cat.idx = c.model_idx
)
INSERT INTO ai_requests
  (id, request_id, user_id, context_id, provider, model, requested_model,
   tokens_used, input_tokens, output_tokens, cost_microdollars,
   cache_read_tokens, cache_hit, latency_ms, is_streaming, status,
   actor_kind, actor_id, created_at, updated_at, completed_at)
SELECT
  'rptseed-req-' || seq,
  'rptseed-req-' || seq,
  user_id,
  -- context_id became NOT NULL in core 0.29.0. These rows are metering
  -- history with no conversation behind them, so they carry the same
  -- "LEGACY" sentinel the demo-organizations migrations use, which the
  -- analytics layer already reads as "no context".
  '00000000-0000-0000-0000-4c4547414359',
  provider,
  model,
  model,
  input_tokens + output_tokens,
  input_tokens,
  output_tokens,
  ((input_tokens::BIGINT * in_rate) / 1000000)
    + ((output_tokens::BIGINT * out_rate) / 1000000),
  cache_read_tokens,
  cache_read_tokens > 0,
  (300 + random() * 8000)::INT,
  random() < 0.4,
  -- A believable failure rate, so the success-rate tile is not a flat 100%.
  CASE WHEN random() < 0.018 THEN 'failed' ELSE 'success' END,
  'user',
  user_id,
  ts,
  ts,
  ts
FROM priced;
SQL

ROWS=$(psql "$DB_URL" -t -A -c "SELECT COUNT(*) FROM ai_requests WHERE id LIKE 'rptseed-%';")
pass "$ROWS ai_requests rows across 6 complete months"
echo ""

info "Open the reports:"
info "  $ADMIN_URL/admin/reports/internal"
info "  $ADMIN_URL/admin/reports/customer?org=astound-digital"
info "Reset with: demo/reports/02-unseed-report-data.sh"
