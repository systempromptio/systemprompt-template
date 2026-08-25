-- Budget warnings gain a kind, so one table records both threshold events the
-- gateway guard observes: 'soft_cap' (month-to-date spend crossed the plan's
-- warning threshold) and 'forecast_overrun' (linear month-end projection first
-- exceeded the hard cap). The primary key widens to include the kind — each
-- event alerts on its own first crossing per organization per month.
ALTER TABLE org_budget_warnings
    ADD COLUMN IF NOT EXISTS kind TEXT NOT NULL DEFAULT 'soft_cap';

ALTER TABLE org_budget_warnings
    DROP CONSTRAINT IF EXISTS org_budget_warnings_kind_check;
ALTER TABLE org_budget_warnings
    ADD CONSTRAINT org_budget_warnings_kind_check
    CHECK (kind IN ('soft_cap', 'forecast_overrun'));

ALTER TABLE org_budget_warnings
    DROP CONSTRAINT IF EXISTS org_budget_warnings_pkey;
ALTER TABLE org_budget_warnings
    ADD PRIMARY KEY (org_id, month, kind);
