-- ============================================================================
-- ARTIFACT PARTS - A2A Protocol Artifact Parts Table
-- Stores the parts of each artifact (reuses A2A spec 6.5 Part union type)
-- ============================================================================

CREATE TABLE IF NOT EXISTS artifact_parts (
    id SERIAL PRIMARY KEY,

    -- Foreign keys (use context_id instead of task_uuid for FK)
    artifact_id TEXT NOT NULL,
    context_id TEXT NOT NULL,

    -- Part type discrimination
    part_kind TEXT NOT NULL CHECK (part_kind IN ('text', 'file', 'data')),

    -- Order within the artifact
    sequence_number INTEGER NOT NULL,

    -- TextPart content (when part_kind = 'text')
    text_content TEXT,

    -- FilePart content (when part_kind = 'file')
    file_name TEXT,
    file_mime_type TEXT,
    file_uri TEXT,
    file_bytes TEXT,

    -- DataPart content (when part_kind = 'data')
    data_content JSONB,

    -- Optional metadata for any part type
    metadata JSONB DEFAULT '{}',

    -- FK now references (context_id, artifact_id) in task_artifacts
    FOREIGN KEY (context_id, artifact_id) REFERENCES task_artifacts(context_id, artifact_id) ON DELETE CASCADE,
    UNIQUE(artifact_id, sequence_number),

    -- Ensure only relevant fields are populated based on part_kind
    CONSTRAINT check_text_part
        CHECK (part_kind != 'text' OR text_content IS NOT NULL),
    CONSTRAINT check_file_part
        CHECK (part_kind != 'file' OR (file_uri IS NOT NULL OR file_bytes IS NOT NULL)),
    CONSTRAINT check_data_part
        CHECK (part_kind != 'data' OR data_content IS NOT NULL)
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_artifact_parts_artifact_id ON artifact_parts(artifact_id);
CREATE INDEX IF NOT EXISTS idx_artifact_parts_context_id ON artifact_parts(context_id);
CREATE INDEX IF NOT EXISTS idx_artifact_parts_kind ON artifact_parts(part_kind);
CREATE INDEX IF NOT EXISTS idx_artifact_parts_sequence ON artifact_parts(artifact_id, sequence_number);