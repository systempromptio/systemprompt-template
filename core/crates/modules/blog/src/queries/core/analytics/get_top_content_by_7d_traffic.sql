SELECT
    mc.id as content_id,
    mc.title,
    mc.slug,
    mc.source_id,
    mc.published_at,
    EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - mc.published_at))/86400 as days_old,
    COUNT(DISTINCT CASE WHEN u.id IS NOT NULL AND u.is_bot = FALSE AND u.is_scanner = FALSE THEN ae.id END) as total_views,
    COUNT(DISTINCT CASE WHEN u.id IS NOT NULL AND u.is_bot = FALSE AND u.is_scanner = FALSE THEN ae.user_id END) as visitors_all_time,
    COUNT(DISTINCT CASE
        WHEN u.id IS NOT NULL
            AND u.is_bot = FALSE
            AND u.is_scanner = FALSE
            AND ae.timestamp >= CURRENT_TIMESTAMP - INTERVAL '1 day'
        THEN ae.user_id
    END) as visitors_1d,
    COUNT(DISTINCT CASE
        WHEN u.id IS NOT NULL
            AND u.is_bot = FALSE
            AND u.is_scanner = FALSE
            AND ae.timestamp >= CURRENT_TIMESTAMP - INTERVAL '7 days'
        THEN ae.user_id
    END) as visitors_7d,
    COUNT(DISTINCT CASE
        WHEN u.id IS NOT NULL
            AND u.is_bot = FALSE
            AND u.is_scanner = FALSE
            AND ae.timestamp >= CURRENT_TIMESTAMP - INTERVAL '30 days'
        THEN ae.user_id
    END) as visitors_30d
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
HAVING COUNT(DISTINCT CASE WHEN u.id IS NOT NULL AND u.is_bot = FALSE AND u.is_scanner = FALSE THEN ae.user_id END) > 0

UNION ALL

SELECT
    'home' as content_id,
    'Home Page' as title,
    '' as slug,
    'home' as source_id,
    NULL::timestamp as published_at,
    0::double precision as days_old,
    COUNT(DISTINCT CASE WHEN u.id IS NOT NULL AND u.is_bot = FALSE AND u.is_scanner = FALSE THEN ae.id END) as total_views,
    COUNT(DISTINCT CASE WHEN u.id IS NOT NULL AND u.is_bot = FALSE AND u.is_scanner = FALSE THEN ae.user_id END) as visitors_all_time,
    COUNT(DISTINCT CASE
        WHEN u.id IS NOT NULL
            AND u.is_bot = FALSE
            AND u.is_scanner = FALSE
            AND ae.timestamp >= CURRENT_TIMESTAMP - INTERVAL '1 day'
        THEN ae.user_id
    END) as visitors_1d,
    COUNT(DISTINCT CASE
        WHEN u.id IS NOT NULL
            AND u.is_bot = FALSE
            AND u.is_scanner = FALSE
            AND ae.timestamp >= CURRENT_TIMESTAMP - INTERVAL '7 days'
        THEN ae.user_id
    END) as visitors_7d,
    COUNT(DISTINCT CASE
        WHEN u.id IS NOT NULL
            AND u.is_bot = FALSE
            AND u.is_scanner = FALSE
            AND ae.timestamp >= CURRENT_TIMESTAMP - INTERVAL '30 days'
        THEN ae.user_id
    END) as visitors_30d
FROM analytics_events ae
LEFT JOIN users u ON ae.user_id = u.id
WHERE ae.event_type = 'page_view'
    AND ae.event_category = 'content'
    AND ae.endpoint = 'GET /'
    AND ae.timestamp >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
GROUP BY content_id, title, slug, source_id, published_at, days_old
HAVING COUNT(DISTINCT CASE WHEN u.id IS NOT NULL AND u.is_bot = FALSE AND u.is_scanner = FALSE THEN ae.user_id END) > 0

ORDER BY visitors_7d DESC, visitors_all_time DESC
