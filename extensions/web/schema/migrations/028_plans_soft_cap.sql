-- Soft spend threshold beside the hard cap. NULL means no warning threshold.
-- Warn-only: the gateway guard records a crossing in org_budget_warnings and
-- logs, but never denies on it. Authored as monthly_cost_warn_usd in
-- services/access-control/plans.yaml; the loader validates warn < cap.
-- Declarative shape lives in 16_organizations.sql.

ALTER TABLE plans
    ADD COLUMN IF NOT EXISTS monthly_cost_warn_microdollars BIGINT;
