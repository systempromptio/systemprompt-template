CREATE UNIQUE INDEX IF NOT EXISTS plugin_usage_events_owner_id
    ON plugin_usage_events(user_id,id);
CREATE UNIQUE INDEX IF NOT EXISTS mcp_tool_executions_owner_id
    ON mcp_tool_executions(user_id,mcp_execution_id);
CREATE UNIQUE INDEX IF NOT EXISTS managed_revisions_owner_id
    ON managed_revisions(owner_id,id);

DO $$ BEGIN
    ALTER TABLE reviewed_production_failures
        ADD CONSTRAINT reviewed_failure_invocation_owner
        FOREIGN KEY(owner_id,invocation_id) REFERENCES plugin_usage_events(user_id,id);
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;
DO $$ BEGIN
    ALTER TABLE reviewed_production_failures
        ADD CONSTRAINT reviewed_failure_case_owner
        FOREIGN KEY(owner_id,development_case_revision_id) REFERENCES managed_revisions(owner_id,id);
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;
DO $$ BEGIN
    ALTER TABLE independently_metered_tool_charges
        ADD CONSTRAINT independently_metered_tool_owner
        FOREIGN KEY(owner_id,mcp_execution_id) REFERENCES mcp_tool_executions(user_id,mcp_execution_id);
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;
