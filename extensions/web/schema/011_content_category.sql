-- Add category column for content filtering (e.g., announcement, guide, article)
-- This is separate from category_id which refers to the source category

ALTER TABLE markdown_content
ADD COLUMN IF NOT EXISTS category TEXT;

CREATE INDEX IF NOT EXISTS idx_markdown_content_category_filter
ON markdown_content(category);
