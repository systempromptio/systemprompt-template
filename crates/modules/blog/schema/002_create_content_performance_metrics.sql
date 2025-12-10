-- ============================================================================
-- Content Performance Metrics - Cumulative views and engagement tracking
-- ============================================================================
-- Purpose: Track stable, monotonically-increasing view counts that never
--          decrease. Prevents the issue where views drop as events age out
--          of time-windows (e.g., 30-day rolling window).
-- ============================================================================

CREATE TABLE IF NOT EXISTS content_performance_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_id TEXT NOT NULL,
    source_id TEXT NOT NULL,

    -- Cumulative counters (NEVER decrease)
    total_views BIGINT NOT NULL DEFAULT 0,
    total_unique_visitors BIGINT NOT NULL DEFAULT 0,
    total_sessions BIGINT NOT NULL DEFAULT 0,

    -- Time-based aggregates (updated hourly via cron)
    views_7d INTEGER NOT NULL DEFAULT 0,
    views_30d INTEGER NOT NULL DEFAULT 0,
    views_90d INTEGER NOT NULL DEFAULT 0,

    -- Engagement metrics
    avg_time_on_page_seconds INTEGER DEFAULT 0,
    bounce_rate DOUBLE PRECISION DEFAULT 0.0,

    -- Tracking timestamps
    first_view_at TIMESTAMPTZ,
    last_view_at TIMESTAMPTZ,
    last_calculated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,

    -- Constraints
    CONSTRAINT fk_content
        FOREIGN KEY (content_id)
        REFERENCES markdown_content(id)
        ON DELETE CASCADE,

    CONSTRAINT check_positive_views
        CHECK (total_views >= 0),

    CONSTRAINT check_positive_visitors
        CHECK (total_unique_visitors >= 0),

    CONSTRAINT check_positive_sessions
        CHECK (total_sessions >= 0),

    UNIQUE(content_id)
);

-- Indexes for fast queries
CREATE INDEX IF NOT EXISTS idx_content_metrics_content_id
ON content_performance_metrics(content_id);

CREATE INDEX IF NOT EXISTS idx_content_metrics_total_views
ON content_performance_metrics(total_views DESC);

CREATE INDEX IF NOT EXISTS idx_content_metrics_views_7d
ON content_performance_metrics(views_7d DESC);

CREATE INDEX IF NOT EXISTS idx_content_metrics_views_30d
ON content_performance_metrics(views_30d DESC);

CREATE INDEX IF NOT EXISTS idx_content_metrics_last_view
ON content_performance_metrics(last_view_at DESC);

CREATE INDEX IF NOT EXISTS idx_content_metrics_updated
ON content_performance_metrics(last_calculated_at DESC);

-- Materialized view for top content (refreshed hourly)
DROP MATERIALIZED VIEW IF EXISTS mv_top_content CASCADE;
CREATE MATERIALIZED VIEW mv_top_content AS
SELECT
    mc.id,
    mc.slug,
    mc.title,
    mc.published_at,
    COALESCE(cpm.total_views, 0) as total_views,
    COALESCE(cpm.total_unique_visitors, 0) as total_unique_visitors,
    COALESCE(cpm.views_7d, 0) as views_7d,
    COALESCE(cpm.views_30d, 0) as views_30d,
    COALESCE(cpm.last_view_at, mc.published_at) as last_view_at
FROM markdown_content mc
LEFT JOIN content_performance_metrics cpm ON mc.id = cpm.content_id
WHERE mc.published_at <= CURRENT_TIMESTAMP
ORDER BY COALESCE(cpm.total_views, 0) DESC;

CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_top_content_id
ON mv_top_content(id);

-- View for content performance details
DROP VIEW IF EXISTS v_content_performance CASCADE;
CREATE VIEW v_content_performance AS
SELECT
    mc.id,
    mc.slug,
    mc.title,
    mc.source_id,
    mc.published_at,
    COALESCE(cpm.total_views, 0) as total_views,
    COALESCE(cpm.total_unique_visitors, 0) as total_unique_visitors,
    COALESCE(cpm.total_sessions, 0) as total_sessions,
    COALESCE(cpm.views_7d, 0) as views_7d,
    COALESCE(cpm.views_30d, 0) as views_30d,
    COALESCE(cpm.views_90d, 0) as views_90d,
    COALESCE(cpm.avg_time_on_page_seconds, 0) as avg_time_on_page_seconds,
    COALESCE(cpm.bounce_rate, 0.0) as bounce_rate,
    COALESCE(cpm.first_view_at, mc.published_at) as first_view_at,
    cpm.last_view_at,
    cpm.last_calculated_at
FROM markdown_content mc
LEFT JOIN content_performance_metrics cpm ON mc.id = cpm.content_id
WHERE mc.published_at <= CURRENT_TIMESTAMP;

-- Trigger function to update cumulative metrics on analytics event insert
CREATE OR REPLACE FUNCTION update_content_metrics()
RETURNS TRIGGER AS $$
DECLARE
    v_content_id TEXT;
BEGIN
    -- Only process page_view events for content
    IF NEW.event_type = 'page_view' AND NEW.event_category = 'content' THEN
        -- Try to match content by slug from endpoint
        -- Examples: 'GET /blog/my-post' or 'GET /my-page'
        v_content_id := (
            SELECT id FROM markdown_content
            WHERE endpoint LIKE '%' || slug || '%'
               OR endpoint = 'GET /blog/' || slug
               OR endpoint = 'GET /' || slug
            LIMIT 1
        );

        IF v_content_id IS NOT NULL THEN
            -- Upsert metrics record, incrementing view count
            INSERT INTO content_performance_metrics (
                content_id,
                source_id,
                total_views,
                total_unique_visitors,
                total_sessions,
                first_view_at,
                last_view_at,
                last_calculated_at
            )
            VALUES (
                v_content_id,
                (SELECT source_id FROM markdown_content WHERE id = v_content_id),
                1,
                1,
                1,
                NEW.timestamp,
                NEW.timestamp,
                CURRENT_TIMESTAMP
            )
            ON CONFLICT (content_id) DO UPDATE SET
                total_views = content_performance_metrics.total_views + 1,
                last_view_at = NEW.timestamp,
                last_calculated_at = CURRENT_TIMESTAMP;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Attach trigger to analytics_events
DROP TRIGGER IF EXISTS trg_update_content_metrics ON analytics_events;
CREATE TRIGGER trg_update_content_metrics
AFTER INSERT ON analytics_events
FOR EACH ROW
WHEN (NEW.event_type = 'page_view' AND NEW.event_category = 'content')
EXECUTE FUNCTION update_content_metrics();

-- Helper function to refresh materialized view concurrently
CREATE OR REPLACE FUNCTION refresh_top_content_view()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_top_content;
END;
$$ LANGUAGE plpgsql;
