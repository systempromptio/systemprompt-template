CREATE TABLE IF NOT EXISTS soul_memories (
    id TEXT PRIMARY KEY,

    -- Memory classification
    memory_type TEXT NOT NULL,
    category TEXT NOT NULL,

    -- Content
    subject TEXT NOT NULL,
    content TEXT NOT NULL,
    context_text TEXT,

    -- Metadata
    priority INTEGER DEFAULT 50,
    confidence REAL DEFAULT 1.0,
    source_task_id TEXT,
    source_context_id TEXT,
    tags TEXT[],
    metadata JSONB DEFAULT '{}'::jsonb,

    -- Lifecycle
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_accessed_at TIMESTAMPTZ,
    access_count INTEGER DEFAULT 0,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT TRUE
);

CREATE INDEX IF NOT EXISTS idx_soul_memories_type ON soul_memories(memory_type);
CREATE INDEX IF NOT EXISTS idx_soul_memories_category ON soul_memories(category);
CREATE INDEX IF NOT EXISTS idx_soul_memories_subject ON soul_memories(subject);
CREATE INDEX IF NOT EXISTS idx_soul_memories_priority ON soul_memories(priority DESC) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_soul_memories_active ON soul_memories(is_active, memory_type) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_soul_memories_tags ON soul_memories USING GIN(tags);
CREATE INDEX IF NOT EXISTS idx_soul_memories_expires ON soul_memories(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_soul_memories_created ON soul_memories(created_at DESC);

CREATE OR REPLACE VIEW v_soul_context AS
SELECT
    memory_type,
    category,
    subject,
    context_text,
    priority
FROM soul_memories
WHERE is_active = TRUE
  AND context_text IS NOT NULL
  AND (expires_at IS NULL OR expires_at > NOW())
ORDER BY
    CASE memory_type
        WHEN 'core' THEN 1
        WHEN 'long_term' THEN 2
        WHEN 'short_term' THEN 3
        WHEN 'working' THEN 4
    END,
    priority DESC;
