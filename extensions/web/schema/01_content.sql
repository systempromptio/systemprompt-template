-- Consolidated schema: Content management tables

CREATE TABLE IF NOT EXISTS markdown_content (
    id TEXT PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    body TEXT NOT NULL,
    author TEXT NOT NULL,
    published_at TIMESTAMPTZ NOT NULL,
    keywords TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'article',
    image TEXT,
    image_optimization_status TEXT,
    category_id TEXT,
    category TEXT,
    source_id TEXT NOT NULL,
    version_hash TEXT NOT NULL,
    public BOOLEAN NOT NULL DEFAULT true,
    links JSONB NOT NULL DEFAULT '[]'::jsonb,
    after_reading_this JSONB NOT NULL DEFAULT '[]'::jsonb,
    related_playbooks JSONB NOT NULL DEFAULT '[]'::jsonb,
    related_code JSONB NOT NULL DEFAULT '[]'::jsonb,
    related_docs JSONB NOT NULL DEFAULT '[]'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Extension columns (ADD IF NOT EXISTS for when core creates the table first)
ALTER TABLE markdown_content ADD COLUMN IF NOT EXISTS image_optimization_status TEXT;
ALTER TABLE markdown_content ADD COLUMN IF NOT EXISTS category TEXT;
ALTER TABLE markdown_content ADD COLUMN IF NOT EXISTS after_reading_this JSONB NOT NULL DEFAULT '[]'::jsonb;
ALTER TABLE markdown_content ADD COLUMN IF NOT EXISTS related_playbooks JSONB NOT NULL DEFAULT '[]'::jsonb;
ALTER TABLE markdown_content ADD COLUMN IF NOT EXISTS related_code JSONB NOT NULL DEFAULT '[]'::jsonb;
ALTER TABLE markdown_content ADD COLUMN IF NOT EXISTS related_docs JSONB NOT NULL DEFAULT '[]'::jsonb;

CREATE INDEX IF NOT EXISTS idx_markdown_content_category ON markdown_content(category_id);
CREATE INDEX IF NOT EXISTS idx_markdown_content_source ON markdown_content(source_id);
CREATE INDEX IF NOT EXISTS idx_markdown_content_published ON markdown_content(published_at DESC);
CREATE INDEX IF NOT EXISTS idx_markdown_content_slug ON markdown_content(slug);
CREATE INDEX IF NOT EXISTS idx_markdown_content_version_hash ON markdown_content(version_hash);
CREATE INDEX IF NOT EXISTS idx_markdown_content_kind ON markdown_content(kind);
CREATE INDEX IF NOT EXISTS idx_markdown_content_links ON markdown_content USING GIN (links);
CREATE INDEX IF NOT EXISTS idx_markdown_content_public ON markdown_content(public) WHERE public = true;
CREATE INDEX IF NOT EXISTS idx_markdown_content_after_reading_this ON markdown_content USING GIN (after_reading_this);
CREATE INDEX IF NOT EXISTS idx_markdown_content_related_playbooks ON markdown_content USING GIN (related_playbooks);
CREATE INDEX IF NOT EXISTS idx_markdown_content_related_code ON markdown_content USING GIN (related_code);
CREATE INDEX IF NOT EXISTS idx_markdown_content_related_docs ON markdown_content USING GIN (related_docs);
CREATE INDEX IF NOT EXISTS idx_markdown_content_category_filter ON markdown_content(category);

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

CREATE TABLE IF NOT EXISTS content_performance_metrics (
    id TEXT PRIMARY KEY,
    content_id TEXT NOT NULL UNIQUE,
    total_views INTEGER NOT NULL DEFAULT 0,
    unique_visitors INTEGER NOT NULL DEFAULT 0,
    avg_time_on_page_seconds DOUBLE PRECISION NOT NULL DEFAULT 0,
    shares_total INTEGER NOT NULL DEFAULT 0,
    shares_linkedin INTEGER NOT NULL DEFAULT 0,
    shares_twitter INTEGER NOT NULL DEFAULT 0,
    comments_count INTEGER NOT NULL DEFAULT 0,
    search_impressions INTEGER NOT NULL DEFAULT 0,
    search_clicks INTEGER NOT NULL DEFAULT 0,
    avg_search_position REAL,
    views_last_7_days INTEGER NOT NULL DEFAULT 0,
    views_last_30_days INTEGER NOT NULL DEFAULT 0,
    trend_direction TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (content_id) REFERENCES markdown_content(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_content_performance_metrics_content_id ON content_performance_metrics(content_id);
CREATE INDEX IF NOT EXISTS idx_content_performance_metrics_total_views ON content_performance_metrics(total_views DESC);
CREATE INDEX IF NOT EXISTS idx_content_performance_metrics_views_7d ON content_performance_metrics(views_last_7_days DESC);
CREATE INDEX IF NOT EXISTS idx_content_performance_metrics_updated ON content_performance_metrics(updated_at DESC);

CREATE TABLE IF NOT EXISTS markdown_fts (
    content_id TEXT PRIMARY KEY,
    search_vector tsvector,
    FOREIGN KEY (content_id) REFERENCES markdown_content(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_markdown_fts_search ON markdown_fts USING GIN(search_vector);
