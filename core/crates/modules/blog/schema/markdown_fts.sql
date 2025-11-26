-- PostgreSQL full-text search using tsvector
-- Note: This table is optional and used for advanced search features
-- The search functionality can work without it using basic text matching
CREATE TABLE IF NOT EXISTS markdown_fts (
    content_id TEXT PRIMARY KEY,
    search_vector tsvector,
    FOREIGN KEY (content_id) REFERENCES markdown_content(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_markdown_fts_search ON markdown_fts USING GIN(search_vector);
