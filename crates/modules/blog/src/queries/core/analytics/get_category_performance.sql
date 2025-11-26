-- Rewritten to use analytics_events with user-based bot filtering
-- Note: engagement metrics (avg_engagement_score, avg_time_on_page) set to NULL as they require client-side tracking
WITH category_stats AS (
    SELECT
        COALESCE(mcat.name, 'Uncategorized') as category_name,
        COUNT(DISTINCT mc.id) as article_count,
        COUNT(DISTINCT CASE WHEN u.id IS NOT NULL AND u.is_bot = FALSE AND u.is_scanner = FALSE THEN ae.id END) as total_views,
        NULL::float as avg_engagement_score,
        NULL::float as avg_time_on_page
    FROM markdown_content mc
    LEFT JOIN markdown_categories mcat ON mc.category_id = mcat.id
    LEFT JOIN analytics_events ae ON
        ae.event_type = 'page_view'
        AND ae.event_category = 'content'
        AND (
            (mc.source_id = 'blog' AND ae.endpoint = 'GET /blog/' || mc.slug)
            OR (mc.source_id = 'pages' AND ae.endpoint = 'GET /' || mc.slug)
            OR (mc.source_id NOT IN ('blog', 'pages') AND ae.endpoint LIKE '%' || mc.slug || '%')
        )
        AND ae.timestamp >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
    LEFT JOIN users u ON ae.user_id = u.id
    GROUP BY mcat.name
)
SELECT *
FROM category_stats
ORDER BY total_views DESC
