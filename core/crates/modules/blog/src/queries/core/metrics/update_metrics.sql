UPDATE blog_performance_metrics
SET
    total_views = $1,
    unique_visitors = $2,
    avg_time_on_page_seconds = $3,
    shares_total = $4,
    shares_linkedin = $5,
    shares_twitter = $6,
    comments_count = $7,
    search_impressions = $8,
    search_clicks = $9,
    avg_search_position = $10,
    views_last_7_days = $11,
    views_last_30_days = $12,
    trend_direction = $13,
    updated_at = CURRENT_TIMESTAMP
WHERE article_id = $14
