CREATE INDEX IF NOT EXISTS idx_markdown_content_after_reading_this ON markdown_content USING GIN (after_reading_this);
CREATE INDEX IF NOT EXISTS idx_markdown_content_related_playbooks ON markdown_content USING GIN (related_playbooks);
CREATE INDEX IF NOT EXISTS idx_markdown_content_related_code ON markdown_content USING GIN (related_code);
CREATE INDEX IF NOT EXISTS idx_markdown_content_related_docs ON markdown_content USING GIN (related_docs);
CREATE INDEX IF NOT EXISTS idx_markdown_content_category_filter ON markdown_content(category);
