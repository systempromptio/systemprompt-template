-- Migration: Remove unused columns and tags tables
-- Date: 2025-12-09
-- Description: Cleanup dead code - created_at, image_url columns and tags tables

-- Drop columns from markdown_content
ALTER TABLE markdown_content DROP COLUMN IF EXISTS created_at;
ALTER TABLE markdown_content DROP COLUMN IF EXISTS image_url;

-- Drop tags tables (CASCADE handles foreign keys)
DROP TABLE IF EXISTS markdown_content_tags CASCADE;
DROP TABLE IF EXISTS markdown_tags CASCADE;
