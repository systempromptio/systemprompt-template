SELECT
    mc.id as content_id,
    mc.title,
    mc.slug,
    mc.source_id,
    mc.published_at,
    EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - mc.published_at))/86400 as days_old,
    COUNT(DISTINCT CASE WHEN u.id IS NOT NULL AND u.is_bot = FALSE AND u.is_scanner = FALSE THEN ae.id END) as total_views,
    COUNT(DISTINCT CASE WHEN u.id IS NOT NULL AND u.is_bot = FALSE AND u.is_scanner = FALSE THEN ae.user_id END) as unique_visitors
FROM markdown_content mc
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
GROUP BY mc.id, mc.title, mc.slug, mc.source_id, mc.published_at
ORDER BY total_views DESC
LIMIT 20
