-- Add structured metadata columns for documentation content
-- Supports: learning objectives, related playbooks, and related code references

ALTER TABLE markdown_content
ADD COLUMN IF NOT EXISTS after_reading_this JSONB NOT NULL DEFAULT '[]'::jsonb,
ADD COLUMN IF NOT EXISTS related_playbooks JSONB NOT NULL DEFAULT '[]'::jsonb,
ADD COLUMN IF NOT EXISTS related_code JSONB NOT NULL DEFAULT '[]'::jsonb;

-- Create GIN indexes for efficient JSONB queries
CREATE INDEX IF NOT EXISTS idx_markdown_content_after_reading_this
    ON markdown_content USING GIN (after_reading_this);
CREATE INDEX IF NOT EXISTS idx_markdown_content_related_playbooks
    ON markdown_content USING GIN (related_playbooks);
CREATE INDEX IF NOT EXISTS idx_markdown_content_related_code
    ON markdown_content USING GIN (related_code);

COMMENT ON COLUMN markdown_content.after_reading_this IS 'Learning objectives - what readers will understand after reading';
COMMENT ON COLUMN markdown_content.related_playbooks IS 'Links to related playbooks (title, url)';
COMMENT ON COLUMN markdown_content.related_code IS 'Links to related GitHub code with line numbers (title, url)';
