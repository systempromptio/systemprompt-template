-- Web-owned side tables.
--
-- These hold web-specific columns that previously lived on core-owned tables.
-- Each row is keyed 1:1 to its core parent and cascades on delete, so the web
-- extension owns its own columns outright instead of ALTERing a table another
-- extension created. The content side table is 13_content_enrichment.sql,
-- installed only when the content member is linked.

CREATE TABLE IF NOT EXISTS user_profile_ext (
    user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    department TEXT NOT NULL DEFAULT 'Default',
    share_token_version INT NOT NULL DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_user_profile_ext_department ON user_profile_ext(department);
