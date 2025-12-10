CREATE TABLE IF NOT EXISTS files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_path TEXT NOT NULL UNIQUE,
    public_url TEXT NOT NULL,
    mime_type VARCHAR(255) NOT NULL,
    file_size_bytes BIGINT,
    ai_content BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB NOT NULL DEFAULT '{}',
    user_id VARCHAR(255),
    session_id VARCHAR(255),
    trace_id VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX IF NOT EXISTS idx_files_user_id ON files(user_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_files_ai_content ON files(ai_content) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_files_created_at ON files(created_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_files_mime_type ON files(mime_type) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_files_metadata ON files USING gin(metadata);
