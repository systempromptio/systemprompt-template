SELECT
    mc.id as content_id,
    mc.title,
    mc.slug,
    COALESCE(cpm.total_views, 0) as total_views,
    COUNT(DISTINCT lc.id) as outbound_clicks,
    COUNT(DISTINCT lc.session_id) as unique_clickers,
    ROUND(
        100.0 * COUNT(DISTINCT lc.id) / NULLIF(COALESCE(cpm.total_views, 0), 0),
        2
    ) as click_through_rate
FROM markdown_content mc
LEFT JOIN content_performance_metrics cpm ON mc.id = cpm.content_id
LEFT JOIN campaign_links cl ON mc.id = cl.source_content_id
LEFT JOIN link_clicks lc ON cl.id = lc.link_id
    AND lc.clicked_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
WHERE mc.published_at <= CURRENT_TIMESTAMP
GROUP BY mc.id, mc.title, mc.slug, cpm.total_views
HAVING COUNT(DISTINCT lc.id) > 0
ORDER BY outbound_clicks DESC
LIMIT 20
