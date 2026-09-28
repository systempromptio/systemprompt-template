-- Cost attribution stamped at request time.
--
-- Spend used to be attributed by joining `ai_requests` to each person's
-- *current* primary group and project, so moving someone silently rewrote
-- their history. This table pins the answer per request: an `AFTER INSERT`
-- statement trigger on core's `ai_requests` copies the primary group and project from
-- `user_scope_defaults` the moment the row lands, and nothing later touches it.
-- `source` says who decided — `primary` is the person's default; `header` is
-- reserved for a per-request override a gateway seam may supply later.
--
-- Extension-only: `ai_requests` is core's and has no scope column, and the
-- insert is a plain statement on the write pool, so the trigger resolves the
-- scope itself from each new row's `user_id`. The body is wrapped so that a
-- fault here warns and returns; it never rolls back the core insert.
-- Established databases are backfilled by migration 087.

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
