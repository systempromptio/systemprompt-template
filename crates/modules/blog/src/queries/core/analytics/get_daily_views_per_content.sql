SELECT
    mc.id as content_id,
    mc.title,
    mc.slug,
    TO_CHAR(DATE(ae.timestamp), 'YYYY-MM-DD') as view_date,
    COUNT(DISTINCT CASE WHEN u.id IS NOT NULL AND u.is_bot = FALSE AND u.is_scanner = FALSE THEN ae.id END) as daily_views
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
WHERE mc.published_at IS NOT NULL
GROUP BY mc.id, mc.title, mc.slug, DATE(ae.timestamp)
HAVING COUNT(DISTINCT CASE WHEN u.id IS NOT NULL AND u.is_bot = FALSE AND u.is_scanner = FALSE THEN ae.id END) > 0
ORDER BY DATE(ae.timestamp) ASC, daily_views DESC
