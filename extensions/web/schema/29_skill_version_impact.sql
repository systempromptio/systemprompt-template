-- Current version-impact schema for clean installations. The trigger and view are
-- created by migration 065 after Core has installed marketplace migrations.
CREATE UNIQUE INDEX IF NOT EXISTS plugin_usage_events_owner_id
    ON plugin_usage_events(user_id,id);
CREATE UNIQUE INDEX IF NOT EXISTS mcp_tool_executions_owner_id
    ON mcp_tool_executions(user_id,mcp_execution_id);
CREATE UNIQUE INDEX IF NOT EXISTS managed_revisions_owner_id
    ON managed_revisions(owner_id,id);

-- The owner-paired keys need the unique indexes declared above, which on an
-- established database only arrive through migration 057. The installer
-- applies every FOREIGN KEY after migrations and indexes (core's deferred
-- foreign-key phase), and skips a key 057 already created under the same
-- columns — so the same declaration converges fresh and upgraded databases.
-- An inline key that ran in the structural phase took the 0.52 preview down
-- at boot on 2026-09-14; this shape is what the upgrade gate proves.
CREATE TABLE IF NOT EXISTS reviewed_production_failures (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL REFERENCES users(id),
    invocation_id TEXT NOT NULL REFERENCES plugin_usage_events(id),
    reviewer_id TEXT NOT NULL REFERENCES users(id),
    sanitized_evidence JSONB NOT NULL,
    development_case_revision_id TEXT NOT NULL REFERENCES managed_revisions(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(owner_id,invocation_id),
    CONSTRAINT reviewed_failure_invocation_owner
        FOREIGN KEY(owner_id,invocation_id) REFERENCES plugin_usage_events(user_id,id),
    CONSTRAINT reviewed_failure_case_owner
        FOREIGN KEY(owner_id,development_case_revision_id) REFERENCES managed_revisions(owner_id,id)
);

