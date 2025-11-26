SELECT
    id, content_id, total_views, unique_visitors, avg_time_on_page_seconds,
    shares_total, shares_linkedin, shares_twitter, comments_count,
    search_impressions, search_clicks, avg_search_position,
    views_last_7_days, views_last_30_days, trend_direction,
    created_at, updated_at
FROM content_performance_metrics
WHERE content_id = $1
