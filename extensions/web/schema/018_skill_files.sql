CREATE TABLE IF NOT EXISTS skill_files (
    id TEXT PRIMARY KEY,
    skill_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'script',
    language TEXT NOT NULL DEFAULT '',
    executable BOOLEAN NOT NULL DEFAULT false,
    size_bytes BIGINT NOT NULL DEFAULT 0,
    checksum TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(skill_id, file_path)
);
CREATE INDEX IF NOT EXISTS idx_skill_files_skill ON skill_files(skill_id);
CREATE INDEX IF NOT EXISTS idx_skill_files_category ON skill_files(skill_id, category);
