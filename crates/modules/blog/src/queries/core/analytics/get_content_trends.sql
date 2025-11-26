SELECT
    mc.id as content_id,
    mc.title,
    mc.slug,
    mc.source_id,
    mc.published_at,
    EXTRACT(DAY FROM CURRENT_TIMESTAMP - mc.published_at::timestamp) as days_since_publish,
    COUNT(DISTINCT CASE
        WHEN ae.timestamp >= CURRENT_TIMESTAMP - INTERVAL '7 days'
             AND u.id IS NOT NULL AND u.is_bot = FALSE AND u.is_scanner = FALSE
        THEN ae.id
    END) as views_7d,
    COUNT(DISTINCT CASE
        WHEN ae.timestamp >= CURRENT_TIMESTAMP - INTERVAL '30 days'
             AND u.id IS NOT NULL AND u.is_bot = FALSE AND u.is_scanner = FALSE
        THEN ae.id
    END) as views_30d
FROM markdown_content mc
LEFT JOIN analytics_events ae ON
    ae.event_type = 'page_view'
    AND ae.event_category = 'content'
    AND (
        (mc.source_id = 'blog' AND ae.endpoint = 'GET /blog/' || mc.slug)
        OR (mc.source_id = 'pages' AND ae.endpoint = 'GET /' || mc.slug)
        OR (mc.source_id NOT IN ('blog', 'pages') AND ae.endpoint LIKE '%' || mc.slug || '%')
    )
    AND ae.timestamp >= CURRENT_TIMESTAMP - INTERVAL '30 days'
LEFT JOIN users u ON ae.user_id = u.id
GROUP BY mc.id, mc.title, mc.slug, mc.source_id, mc.published_at
ORDER BY views_7d DESC
LIMIT 50
