-- ============================================================================
-- TASK MESSAGES - A2A Protocol Message History Table (ENHANCED with Analytics)
-- ============================================================================

CREATE TABLE IF NOT EXISTS task_messages (
    id SERIAL PRIMARY KEY,

    -- Foreign key to task
    task_id TEXT NOT NULL,

    -- A2A Message required fields
    message_id TEXT NOT NULL,
    client_message_id TEXT,
    role TEXT NOT NULL CHECK (role IN ('user', 'agent')),

    -- Optional fields from A2A Message
    context_id TEXT,

    -- Analytics Context (promoted from metadata for fast queries)
    user_id TEXT,
    session_id TEXT,
    trace_id TEXT,

    -- Timestamps for ordering
    sequence_number INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    -- Metadata for A2A protocol extensions only
    metadata JSONB DEFAULT '{}',

    -- A2A Protocol: Reference task IDs for context
    reference_task_ids TEXT[],

    FOREIGN KEY (task_id) REFERENCES agent_tasks(task_id) ON DELETE CASCADE,
    UNIQUE(task_id, message_id),       -- For queries by task
    UNIQUE(message_id, task_id),       -- For FK from message_parts (must match FK column order)
    UNIQUE(task_id, sequence_number)
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_task_messages_task_id ON task_messages(task_id);
CREATE INDEX IF NOT EXISTS idx_task_messages_message_id ON task_messages(message_id);
CREATE INDEX IF NOT EXISTS idx_task_messages_sequence ON task_messages(task_id, sequence_number);
CREATE INDEX IF NOT EXISTS idx_task_messages_client_id ON task_messages(client_message_id) WHERE client_message_id IS NOT NULL;

-- Analytics Indexes
CREATE INDEX IF NOT EXISTS idx_task_messages_user_id ON task_messages(user_id);
CREATE INDEX IF NOT EXISTS idx_task_messages_session_id ON task_messages(session_id);
CREATE INDEX IF NOT EXISTS idx_task_messages_trace_id ON task_messages(trace_id);