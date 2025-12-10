-- ============================================================================
-- TASKS - A2A Protocol Task Table (ENHANCED with Analytics)
-- ============================================================================

CREATE TABLE IF NOT EXISTS agent_tasks (
    -- Primary key as task_id (A2A spec requirement)
    task_id TEXT PRIMARY KEY NOT NULL,

    -- A2A Task required fields (exact spec compliance)
    context_id TEXT NOT NULL,

    -- A2A TaskStatus fields
    status TEXT NOT NULL DEFAULT 'submitted' CHECK (
        status IN (
            'submitted', 'working', 'input-required', 'completed',
            'canceled', 'failed', 'rejected', 'auth-required', 'unknown'
        )
    ),
    status_timestamp TIMESTAMPTZ,

    -- Analytics Context (promoted from metadata for fast queries)
    user_id TEXT,
    session_id TEXT,
    trace_id TEXT,
    agent_name TEXT,

    -- Task execution timing
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    execution_time_ms INTEGER,

    -- A2A Task optional metadata field (JSON for A2A protocol extensions only)
    metadata JSONB DEFAULT '{}',

    -- Standard audit fields
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,

    -- Foreign key constraints
    FOREIGN KEY (context_id) REFERENCES user_contexts(context_id) ON DELETE CASCADE
);

-- Indexes for common A2A protocol queries
CREATE INDEX IF NOT EXISTS idx_agent_tasks_context_id ON agent_tasks(context_id);
CREATE INDEX IF NOT EXISTS idx_agent_tasks_status ON agent_tasks(status);
CREATE INDEX IF NOT EXISTS idx_agent_tasks_status_timestamp ON agent_tasks(status_timestamp);
CREATE INDEX IF NOT EXISTS idx_agent_tasks_created_at ON agent_tasks(created_at);
CREATE INDEX IF NOT EXISTS idx_agent_tasks_updated_at ON agent_tasks(updated_at);

-- Compound index for context + state queries
CREATE INDEX IF NOT EXISTS idx_agent_tasks_context_status ON agent_tasks(context_id, status);

-- Analytics Indexes
CREATE INDEX IF NOT EXISTS idx_agent_tasks_user_id ON agent_tasks(user_id);
CREATE INDEX IF NOT EXISTS idx_agent_tasks_session_id ON agent_tasks(session_id);
CREATE INDEX IF NOT EXISTS idx_agent_tasks_trace_id ON agent_tasks(trace_id);
CREATE INDEX IF NOT EXISTS idx_agent_tasks_user_created ON agent_tasks(user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_agent_tasks_agent_name ON agent_tasks(agent_name);

-- Execution Time Indexes
CREATE INDEX IF NOT EXISTS idx_agent_tasks_started_at ON agent_tasks(started_at DESC);
CREATE INDEX IF NOT EXISTS idx_agent_tasks_completed_at ON agent_tasks(completed_at DESC);
CREATE INDEX IF NOT EXISTS idx_agent_tasks_execution_time ON agent_tasks(execution_time_ms DESC);

-- Trigger to update timestamp
DROP TRIGGER IF EXISTS update_agent_tasks_updated_at ON agent_tasks;
CREATE TRIGGER update_agent_tasks_updated_at
    BEFORE UPDATE ON agent_tasks
    FOR EACH ROW
    EXECUTE FUNCTION update_timestamp_trigger();