-- Composite indexes matching the analytics/trace query shapes.
--
-- plugin_usage_events is this extension's table: the trace CTEs filter on a
-- created_at range and group by session_id, which is exactly the composite's
-- shape; the old single-column session_id index is covered by its leftmost
-- column and dropped.
--
-- governance_decisions is CORE-owned. No web migration touched a core table
-- before this one; the exception is deliberate: the per-policy rollups and
-- windowed rankings in this repo scan by created_at then filter by policy,
-- and core does not ship the composite. CREATE INDEX IF NOT EXISTS is
-- additive and idempotent, so a later core-side declarative index converges
-- harmlessly. Guarded with to_regclass because extension migration order
-- relative to core schema install is not guaranteed on a fresh database
-- (an established database always has the table, so production converges).
-- Upstream adoption request: add both indexes to core's
-- crates/infra/security/schema/governance_decisions.sql.

CREATE INDEX IF NOT EXISTS idx_plugin_usage_session_created
    ON plugin_usage_events(session_id, created_at);

DROP INDEX IF EXISTS idx_plugin_usage_session;

DO $$ BEGIN
    IF to_regclass('governance_decisions') IS NOT NULL THEN
        CREATE INDEX IF NOT EXISTS idx_governance_decisions_policy_created
            ON governance_decisions(policy, created_at);
        CREATE INDEX IF NOT EXISTS idx_governance_decisions_tool_name
            ON governance_decisions(tool_name);
    END IF;
END $$;
