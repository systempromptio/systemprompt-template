-- Web-owned side tables.
--
-- These hold web-specific columns that previously lived on core-owned tables
-- (`markdown_content`, `users`). Each row is keyed 1:1 to its core parent and
-- cascades on delete, so the web extension owns its own columns outright
-- instead of ALTERing a table another extension created.

CREATE TABLE IF NOT EXISTS markdown_content_enrichment (
    content_id TEXT PRIMARY KEY REFERENCES markdown_content(id) ON DELETE CASCADE,
    category TEXT,
    after_reading_this JSONB NOT NULL DEFAULT '[]'::jsonb,
    related_playbooks JSONB NOT NULL DEFAULT '[]'::jsonb,
    related_code JSONB NOT NULL DEFAULT '[]'::jsonb,
    related_docs JSONB NOT NULL DEFAULT '[]'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_mce_category ON markdown_content_enrichment(category);
CREATE INDEX IF NOT EXISTS idx_mce_after_reading_this ON markdown_content_enrichment USING GIN (after_reading_this);
CREATE INDEX IF NOT EXISTS idx_mce_related_playbooks ON markdown_content_enrichment USING GIN (related_playbooks);
CREATE INDEX IF NOT EXISTS idx_mce_related_code ON markdown_content_enrichment USING GIN (related_code);
CREATE INDEX IF NOT EXISTS idx_mce_related_docs ON markdown_content_enrichment USING GIN (related_docs);

CREATE TABLE IF NOT EXISTS user_profile_ext (
    user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    department TEXT NOT NULL DEFAULT 'Default',
    share_token_version INT NOT NULL DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_user_profile_ext_department ON user_profile_ext(department);
