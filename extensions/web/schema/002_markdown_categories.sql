-- Blog extension: markdown_categories table
-- Hierarchical category structure for organizing content

CREATE TABLE IF NOT EXISTS markdown_categories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    parent_id TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (parent_id) REFERENCES markdown_categories(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_markdown_categories_slug ON markdown_categories(slug);
CREATE INDEX IF NOT EXISTS idx_markdown_categories_parent ON markdown_categories(parent_id);
