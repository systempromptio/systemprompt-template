CREATE TABLE IF NOT EXISTS markdown_content (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL UNIQUE,
    slug TEXT NOT NULL UNIQUE,

    title TEXT NOT NULL,
    description TEXT NOT NULL,
    body TEXT NOT NULL,
    excerpt TEXT NOT NULL,

    author TEXT NOT NULL,
    published_at TIMESTAMP NOT NULL,
    keywords TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'article',
    image TEXT,

    category_id TEXT,
    source_id TEXT NOT NULL,

    version_hash TEXT NOT NULL,

    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_markdown_content_category ON markdown_content(category_id);
CREATE INDEX IF NOT EXISTS idx_markdown_content_source ON markdown_content(source_id);
CREATE INDEX IF NOT EXISTS idx_markdown_content_published ON markdown_content(published_at DESC);
CREATE INDEX IF NOT EXISTS idx_markdown_content_slug ON markdown_content(slug);
CREATE INDEX IF NOT EXISTS idx_markdown_content_version_hash ON markdown_content(version_hash);
CREATE INDEX IF NOT EXISTS idx_markdown_content_kind ON markdown_content(kind);
