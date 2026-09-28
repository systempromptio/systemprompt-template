-- The judge's one label per gateway conversation (a `user_contexts` context),
-- written by the `conversation_judge` job. The job leases pending rows, reads
-- the conversation's transcript, and asks a model — in one structured call —
-- for a title, a summary, a fixed-taxonomy intent category, an outcome, the
-- skills it observed and a single 0–100 `completion` score: did the assistant
-- deliver what was originally asked. A judged row carries the fingerprint of
-- what was read (request count + last request time); a conversation that
-- grows past it is queued again. `trigger` says whether a person asked for
-- the verdict from the console. Twin: migrations/086.

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
