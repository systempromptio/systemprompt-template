-- Migration: Remove parent_content_id column (tech debt cleanup)
-- This column was used for social content linking but is no longer needed

-- Drop the dependent view first
DROP VIEW IF EXISTS content;

-- Drop the index
DROP INDEX IF EXISTS idx_markdown_content_parent;

-- Drop the column
ALTER TABLE markdown_content DROP COLUMN IF EXISTS parent_content_id;

-- Recreate the view without parent_content_id
CREATE OR REPLACE VIEW content AS
SELECT
    id, slug, title, description, body, author, published_at, keywords, kind,
    image, category_id, source_id, version_hash, created_at, updated_at, links, public,
    to_tsvector('english', title || ' ' || description || ' ' || body || ' ' || keywords) AS tsv
FROM markdown_content;
