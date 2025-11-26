-- Migration: Add social content support with parent linking
-- Purpose: Allow social media posts to be linked to parent blog posts
--          with automatic cascade deletion when parent is removed

-- Add parent_content_id column to link social content to blog posts
ALTER TABLE markdown_content
ADD COLUMN IF NOT EXISTS parent_content_id TEXT
REFERENCES markdown_content(id) ON DELETE CASCADE;

-- Add index for efficient querying of social content by parent
CREATE INDEX IF NOT EXISTS idx_markdown_content_parent
ON markdown_content(parent_content_id)
WHERE parent_content_id IS NOT NULL;

-- Add index for content_type queries (social content filtering)
CREATE INDEX IF NOT EXISTS idx_markdown_content_type
ON markdown_content(content_type);

-- Add composite index for common query pattern (parent + type)
CREATE INDEX IF NOT EXISTS idx_markdown_content_parent_type
ON markdown_content(parent_content_id, content_type)
WHERE parent_content_id IS NOT NULL;

-- Add check constraint to ensure social content has parent
-- (Regular content should not have parent_content_id)
ALTER TABLE markdown_content
ADD CONSTRAINT chk_social_has_parent
CHECK (
  (content_type NOT LIKE 'social_%' AND parent_content_id IS NULL) OR
  (content_type LIKE 'social_%' AND parent_content_id IS NOT NULL)
);

-- Comments for documentation
COMMENT ON COLUMN markdown_content.parent_content_id IS
'Links social content to parent blog post. NULL for regular content. CASCADE deletes social content when parent is deleted.';
