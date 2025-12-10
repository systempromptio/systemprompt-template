-- ============================================================================
-- USER CONTEXTS - Conversation Metadata (UI Layer Only)
-- ============================================================================
-- Stores user-friendly names for A2A contextId values.
-- Contexts are cross-agent - a single conversation can span multiple agents.
-- The actual conversation data (tasks, messages, parts) lives in A2A protocol tables.

CREATE TABLE IF NOT EXISTS user_contexts (
    -- Context ID (matches agent_tasks.context_id and task_messages.context_id)
    context_id TEXT PRIMARY KEY NOT NULL,

    -- Owner of this context (supports anonymous users with regular UUIDs)
    user_id TEXT NOT NULL,

    -- Session ID for tracking user sessions
    session_id TEXT,

    -- User-friendly conversation name
    name TEXT NOT NULL,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,

    -- Foreign keys for data protection
    CONSTRAINT fk_user_contexts_session
        FOREIGN KEY (session_id)
        REFERENCES user_sessions(session_id)
        ON DELETE SET NULL,
    CONSTRAINT fk_user_contexts_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE SET NULL
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_user_contexts_user ON user_contexts(user_id);
CREATE INDEX IF NOT EXISTS idx_user_contexts_user_updated ON user_contexts(user_id, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_user_contexts_session ON user_contexts(session_id);

-- Trigger to update timestamp
DROP TRIGGER IF EXISTS update_user_contexts_updated_at ON user_contexts;
CREATE TRIGGER update_user_contexts_updated_at
    BEFORE UPDATE ON user_contexts
    FOR EACH ROW
    EXECUTE FUNCTION update_timestamp_trigger();
