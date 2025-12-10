-- ============================================================================
-- CONVERSATION SUBJECTS - Subject/Topic Analysis for Agent Conversations
-- ============================================================================
CREATE TABLE IF NOT EXISTS conversation_subjects (
    id SERIAL PRIMARY KEY,
    task_id VARCHAR(255) NOT NULL UNIQUE,
    extracted_keywords TEXT,  -- JSON array of strings: ["keyword1", "keyword2"]
    primary_topic VARCHAR(255),  -- Classified primary topic
    topic_confidence DOUBLE PRECISION DEFAULT 0.0,  -- Confidence score 0-1
    analyzed_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (task_id) REFERENCES agent_tasks(task_id) ON DELETE CASCADE
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_conversation_subjects_task_id ON conversation_subjects(task_id);
CREATE INDEX IF NOT EXISTS idx_conversation_subjects_primary_topic ON conversation_subjects(primary_topic);
CREATE INDEX IF NOT EXISTS idx_conversation_subjects_analyzed_at ON conversation_subjects(analyzed_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversation_subjects_confidence ON conversation_subjects(topic_confidence DESC);
