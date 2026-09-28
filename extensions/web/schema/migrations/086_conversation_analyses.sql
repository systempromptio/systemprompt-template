-- The judge's one label per gateway conversation (title, summary, intent
-- category, outcome, one 0-100 completion score), and the per-session view of
-- the skills the hooks reported. Collapses astound's table, its judge columns
-- and the view into one step. Twin of schema/37_conversation_analyses.sql.

CREATE TABLE IF NOT EXISTS conversation_analyses (
    context_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'classified', 'failed')),
    category TEXT CHECK (category IN ('development', 'business-analysis', 'operations',
        'admin-config', 'writing-comms', 'research-learning', 'other')),
    summary TEXT,
    tags TEXT[] NOT NULL DEFAULT '{}',
    skills_used TEXT[] NOT NULL DEFAULT '{}',
    outcome TEXT CHECK (outcome IN ('achieved', 'partial', 'abandoned', 'unclear')),
    confidence REAL CHECK (confidence BETWEEN 0 AND 1),
    provider TEXT,
    model TEXT,
    ai_request_id TEXT,
    source_request_count BIGINT NOT NULL DEFAULT 0,
    source_last_at TIMESTAMPTZ,
    classified_at TIMESTAMPTZ,
    attempts INT NOT NULL DEFAULT 0,
    next_attempt TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    lease_token TEXT,
    lease_until TIMESTAMPTZ,
    last_error TEXT,
    title TEXT,
    completion SMALLINT CHECK (completion BETWEEN 0 AND 100),
    completion_rationale TEXT,
    input_tokens INTEGER,
    output_tokens INTEGER,
    cost_microdollars BIGINT,
    trigger TEXT NOT NULL DEFAULT 'automatic' CHECK (trigger IN ('automatic', 'manual')),
    requested_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);
CREATE INDEX IF NOT EXISTS idx_conversation_analyses_pending
    ON conversation_analyses(next_attempt) WHERE status <> 'classified';
CREATE INDEX IF NOT EXISTS idx_conversation_analyses_category
    ON conversation_analyses(category, classified_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversation_analyses_user
    ON conversation_analyses(user_id, classified_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversation_analyses_skills
    ON conversation_analyses USING gin(skills_used);

-- Skills the harness hooks reported per client session, so a conversation
-- shows what it invoked even before the judge has seen it.
CREATE OR REPLACE VIEW conversation_skill_uses AS
SELECT e.session_id AS client_session_id, e.skill,
       COUNT(*)::bigint AS invocations, MIN(e.invoked_at) AS first_at
FROM analysis_skill_events e
WHERE e.skill IS NOT NULL
GROUP BY e.session_id, e.skill;
