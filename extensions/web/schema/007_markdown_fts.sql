-- Blog extension: markdown_fts table
-- Full-text search index for content

CREATE TABLE IF NOT EXISTS markdown_fts (
    content_id TEXT PRIMARY KEY,
    search_vector tsvector,
    FOREIGN KEY (content_id) REFERENCES markdown_content(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_markdown_fts_search ON markdown_fts USING GIN(search_vector);
