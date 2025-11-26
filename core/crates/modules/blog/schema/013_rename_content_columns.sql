-- Rename content columns for consistency
-- content -> body (the actual markdown body text)
-- content_type -> kind (the type of content: article, page, etc.)

ALTER TABLE markdown_content RENAME COLUMN content TO body;
ALTER TABLE markdown_content RENAME COLUMN content_type TO kind;

-- Update indexes that reference the old column name
DROP INDEX IF EXISTS idx_markdown_content_type;
CREATE INDEX IF NOT EXISTS idx_markdown_content_kind ON markdown_content(kind);

-- Remove version column (no longer used without revisions)
ALTER TABLE markdown_content DROP COLUMN IF EXISTS version;

-- Drop the revisions table (no longer used)
DROP TABLE IF EXISTS markdown_content_revisions CASCADE;
