CREATE TABLE IF NOT EXISTS markdown_content_tags (
    content_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,

    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (content_id, tag_id),
    FOREIGN KEY (content_id) REFERENCES markdown_content(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES markdown_tags(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_markdown_content_tags_tag ON markdown_content_tags(tag_id);
