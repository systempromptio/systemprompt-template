CREATE TABLE IF NOT EXISTS content_files (
    id SERIAL PRIMARY KEY,
    content_id TEXT NOT NULL,
    file_id UUID NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'attachment',
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_content_files_content 
        FOREIGN KEY (content_id) REFERENCES markdown_content(id) ON DELETE CASCADE,
    CONSTRAINT fk_content_files_file 
        FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
    CONSTRAINT content_files_unique UNIQUE (content_id, file_id, role)
);

CREATE INDEX IF NOT EXISTS idx_content_files_content_id ON content_files(content_id);
CREATE INDEX IF NOT EXISTS idx_content_files_file_id ON content_files(file_id);
CREATE INDEX IF NOT EXISTS idx_content_files_role ON content_files(role);
