-- Migration: Schema improvements for content management
-- Purpose: Add public flag, remove redundant columns

-- Add public flag to control visibility
ALTER TABLE markdown_content
ADD COLUMN IF NOT EXISTS public BOOLEAN NOT NULL DEFAULT true;

-- Create index for public flag queries
CREATE INDEX IF NOT EXISTS idx_markdown_content_public ON markdown_content(public) WHERE public = true;

-- Remove file_path column (switch to slug-based duplicate detection)
-- First, drop the unique constraint
ALTER TABLE markdown_content
DROP CONSTRAINT IF EXISTS markdown_content_file_path_key;

-- Then drop the column
ALTER TABLE markdown_content
DROP COLUMN IF EXISTS file_path;

-- Remove excerpt column (duplicates description)
-- First drop dependent FTS view and tsv column
DROP VIEW IF EXISTS markdown_fts CASCADE;
ALTER TABLE markdown_content
DROP COLUMN IF EXISTS tsv;

ALTER TABLE markdown_content
DROP COLUMN IF EXISTS excerpt;

-- Recreate FTS column and view without excerpt (use description instead)
ALTER TABLE markdown_content
ADD COLUMN tsv tsvector GENERATED ALWAYS AS (
    to_tsvector('english', coalesce(title, '') || ' ' || coalesce(description, '') || ' ' || coalesce(keywords, '') || ' ' || coalesce(content, ''))
) STORED;

CREATE INDEX IF NOT EXISTS idx_markdown_content_fts ON markdown_content USING gin(tsv);

CREATE OR REPLACE VIEW markdown_fts AS
SELECT id, slug, title, description, source_id, category_id, content_type, tsv
FROM markdown_content;

-- Add comment for documentation
COMMENT ON COLUMN markdown_content.public IS
'Controls whether content is publicly visible. Private content (public=false) is excluded from sitemaps and search results.';
