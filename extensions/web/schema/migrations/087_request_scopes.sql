-- @cost: rows=3644 measured=30s triggers=suspended
-- (Estimate, not a measurement on this repo: rows is the ai_requests count the
-- equivalent production backfill wrote; 30s keeps the derived
-- statement_timeout at the runner's 300s default.)
-- Cost attribution stamped at request time. Twin of
-- schema/40_request_scopes.sql, plus the backfill below.
--
-- The trigger only sees requests inserted after it exists. Every request
-- already on the books is attributed here from the person's primary group
-- and project as they stand today -- the best answer available, and the same
-- one the old membership join gave. From this point on a request's scope is
-- fixed when it lands and moving a person never rewrites it.

CREATE TABLE IF NOT EXISTS ai_request_scopes (
    request_id TEXT PRIMARY KEY REFERENCES ai_requests(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL,
    group_id TEXT REFERENCES groups(id) ON DELETE SET NULL,
    project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
    source TEXT NOT NULL DEFAULT 'primary' CHECK (source IN ('primary', 'header')),
    resolved_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_ai_request_scopes_group
    ON ai_request_scopes(group_id);
CREATE INDEX IF NOT EXISTS idx_ai_request_scopes_project
    ON ai_request_scopes(project_id);
CREATE INDEX IF NOT EXISTS idx_ai_request_scopes_user
    ON ai_request_scopes(user_id);

-- A row is written for every request, including one no primary covers: a
-- NULL group and project is the durable record that the request was
-- unattributed when it happened, which is what lets the remainder bucket be
-- read from this table rather than re-derived. Only a user actor is resolved;
-- a job or MCP actor carries its owner's id and is never a person's spend.
CREATE OR REPLACE FUNCTION request_scope_stamp_ai_requests()
RETURNS TRIGGER AS $$
BEGIN
    BEGIN
        -- TG_LEVEL: the installer swaps function bodies before migrations run,
        -- so a database mid-upgrade still fires the row-form trigger into this
        -- body until the declarative phase replaces it below.
        IF TG_LEVEL = 'ROW' THEN
            INSERT INTO ai_request_scopes (request_id, user_id, group_id, project_id, source)
            SELECT NEW.id, NEW.user_id, d.primary_group_id, d.primary_project_id, 'primary'
              FROM (SELECT 1) one
              LEFT JOIN user_scope_defaults d
                ON NEW.actor_kind = 'user' AND d.user_id = NEW.user_id
            ON CONFLICT (request_id) DO NOTHING;
            RETURN NEW;
        END IF;
        INSERT INTO ai_request_scopes (request_id, user_id, group_id, project_id, source)
        SELECT r.id, r.user_id, d.primary_group_id, d.primary_project_id, 'primary'
          FROM new_rows r
          LEFT JOIN user_scope_defaults d
            ON r.actor_kind = 'user' AND d.user_id = r.user_id
        ON CONFLICT (request_id) DO NOTHING;
    EXCEPTION WHEN OTHERS THEN
        RAISE WARNING 'request_scope_stamp_ai_requests failed: %', SQLERRM;
    END;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Statement-level: one INSERT ... SELECT over the rows the statement added,
-- not one lookup and insert per request. The row form is dropped first
-- because a statement trigger with a transition table is a new definition,
-- not a replacement.
DROP TRIGGER IF EXISTS request_scope_stamp_ai_requests_trg ON ai_requests;
CREATE OR REPLACE TRIGGER request_scope_stamp_ai_requests_stmt
    AFTER INSERT ON ai_requests
    REFERENCING NEW TABLE AS new_rows
    FOR EACH STATEMENT
    EXECUTE FUNCTION request_scope_stamp_ai_requests();


INSERT INTO ai_request_scopes (request_id, user_id, group_id, project_id, source)
SELECT r.id,
       r.user_id,
       CASE WHEN r.actor_kind = 'user' THEN d.primary_group_id END,
       CASE WHEN r.actor_kind = 'user' THEN d.primary_project_id END,
       'primary'
FROM ai_requests r
LEFT JOIN user_scope_defaults d ON d.user_id = r.user_id
ON CONFLICT (request_id) DO NOTHING;
