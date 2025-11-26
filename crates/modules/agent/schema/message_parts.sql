-- ============================================================================
-- MESSAGE PARTS - A2A Protocol Message Parts Table
-- Stores the parts of each message (A2A spec 6.5 Part union type)
-- ============================================================================

CREATE TABLE IF NOT EXISTS message_parts (
    id SERIAL PRIMARY KEY,

    -- Foreign key to message
    message_id TEXT NOT NULL,
    task_id TEXT NOT NULL,

    -- Part type discrimination
    part_kind TEXT NOT NULL CHECK (part_kind IN ('text', 'file', 'data')),

    -- Order within the message
    sequence_number INTEGER NOT NULL,

    -- TextPart content (when part_kind = 'text')
    text_content TEXT,

    -- FilePart content (when part_kind = 'file')
    file_name TEXT,
    file_mime_type TEXT,
    file_uri TEXT, -- URL pointing to file content
    file_bytes TEXT, -- Base64-encoded content (if provided directly)

    -- DataPart content (when part_kind = 'data')
    data_content JSONB, -- JSON object

    -- Optional metadata for any part type
    metadata JSONB DEFAULT '{}',

    FOREIGN KEY (message_id, task_id) REFERENCES task_messages(message_id, task_id) ON DELETE CASCADE,
    UNIQUE(message_id, sequence_number),

    -- Ensure only relevant fields are populated based on part_kind
    CONSTRAINT check_text_part
        CHECK (part_kind != 'text' OR text_content IS NOT NULL),
    CONSTRAINT check_file_part
        CHECK (part_kind != 'file' OR (file_uri IS NOT NULL OR file_bytes IS NOT NULL)),
    CONSTRAINT check_data_part
        CHECK (part_kind != 'data' OR data_content IS NOT NULL)
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_message_parts_message_id ON message_parts(message_id);
CREATE INDEX IF NOT EXISTS idx_message_parts_task_id ON message_parts(task_id);
CREATE INDEX IF NOT EXISTS idx_message_parts_kind ON message_parts(part_kind);
CREATE INDEX IF NOT EXISTS idx_message_parts_sequence ON message_parts(message_id, sequence_number);