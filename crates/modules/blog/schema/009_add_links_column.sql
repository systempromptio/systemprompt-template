ALTER TABLE markdown_content
ADD COLUMN links JSONB DEFAULT '[]'::jsonb;

CREATE INDEX idx_markdown_content_links ON markdown_content USING GIN (links);
