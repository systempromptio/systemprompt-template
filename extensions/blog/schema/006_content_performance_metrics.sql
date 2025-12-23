-- Blog extension: content_performance_metrics table
-- Aggregated performance metrics for content items

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
