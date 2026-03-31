-- Composite index for the rate limit query in governance evaluation
-- Covers: SELECT COUNT(*) FROM governance_decisions
--         WHERE session_id = $1 AND user_id = $2 AND created_at > NOW() - INTERVAL '1 minute'

CREATE INDEX IF NOT EXISTS idx_governance_decisions_rate_limit
    ON governance_decisions(session_id, user_id, created_at DESC);
