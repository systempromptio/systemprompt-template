-- Add related_docs column for links to other documentation pages
-- Keeps internal doc links structured and separate from external links

ALTER TABLE markdown_content
ADD COLUMN IF NOT EXISTS related_docs JSONB NOT NULL DEFAULT '[]'::jsonb;

-- Create GIN index for efficient JSONB queries
CREATE INDEX IF NOT EXISTS idx_markdown_content_related_docs
    ON markdown_content USING GIN (related_docs);

COMMENT ON COLUMN markdown_content.related_docs IS 'Links to related documentation pages (title, url)';
