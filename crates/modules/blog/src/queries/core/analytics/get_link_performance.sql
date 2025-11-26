SELECT
    cl.campaign_name,
    cl.link_type,
    COUNT(DISTINCT cl.id) as total_links,
    COUNT(lc.id) as total_clicks,
    COUNT(DISTINCT lc.session_id) FILTER (WHERE lc.session_id IS NOT NULL) as unique_clicks,
    COALESCE(
        ROUND(
            COUNT(lc.id)::NUMERIC / NULLIF(COUNT(DISTINCT cl.id), 0),
            1
        ),
        0
    ) as avg_clicks_per_link
FROM campaign_links cl
LEFT JOIN link_clicks lc ON cl.id::TEXT = lc.link_id::TEXT
WHERE cl.created_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
GROUP BY cl.campaign_name, cl.link_type
HAVING COUNT(lc.id) > 0
ORDER BY total_clicks DESC, total_links DESC
LIMIT 20
