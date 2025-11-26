-- Migration: Remove social content support
-- Purpose: Clean up social content and parent linking functionality
--          Social content has been removed from the system

-- Delete all social content rows
DELETE FROM markdown_content WHERE content_type LIKE 'social_%';

-- Drop check constraint
ALTER TABLE markdown_content
DROP CONSTRAINT IF EXISTS chk_social_has_parent;

-- Drop indexes related to social content
DROP INDEX IF EXISTS idx_markdown_content_parent;
DROP INDEX IF EXISTS idx_markdown_content_type;
DROP INDEX IF EXISTS idx_markdown_content_parent_type;

-- Drop parent_content_id column
ALTER TABLE markdown_content
DROP COLUMN IF EXISTS parent_content_id;

-- Comments for documentation
COMMENT ON TABLE markdown_content IS
'Stores markdown content without social media linking support';
