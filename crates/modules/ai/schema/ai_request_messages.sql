-- ============================================================================
-- AI REQUEST MESSAGES - Normalized AI conversation messages
-- ============================================================================
--
-- DATA RETENTION: Messages are automatically deleted via CASCADE when parent
-- ai_request is deleted. Follow ai_requests retention policy.
-- ============================================================================
CREATE TABLE IF NOT EXISTS ai_request_messages (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id VARCHAR(255) NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system', 'tool')),
    content TEXT NOT NULL,
    sequence_number INTEGER NOT NULL,
    -- Optional fields
    name VARCHAR(255),              -- For function/tool messages
    tool_call_id VARCHAR(255),      -- For tool response messages
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (request_id) REFERENCES ai_requests(id) ON DELETE CASCADE,
    UNIQUE(request_id, sequence_number)
);
CREATE INDEX IF NOT EXISTS idx_ai_request_messages_request_id ON ai_request_messages(request_id);
CREATE INDEX IF NOT EXISTS idx_ai_request_messages_role ON ai_request_messages(role);
CREATE INDEX IF NOT EXISTS idx_ai_request_messages_sequence ON ai_request_messages(request_id, sequence_number);
