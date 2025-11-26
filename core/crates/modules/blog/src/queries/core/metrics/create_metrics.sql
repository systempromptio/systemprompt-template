INSERT INTO blog_performance_metrics (
    id, article_id, total_views, unique_visitors, avg_time_on_page_seconds,
    shares_total, shares_linkedin, shares_twitter, comments_count,
    search_impressions, search_clicks, avg_search_position,
    views_last_7_days, views_last_30_days, trend_direction
) VALUES (
    $1, $2, $3, $4, $5,
    $6, $7, $8, $9,
    $10, $11, $12,
    $13, $14, $15
)
