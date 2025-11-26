SELECT
    id, short_code, target_url, link_type, campaign_id, campaign_name,
    source_content_id, source_page, utm_params, link_text, link_position,
    destination_type, click_count, unique_click_count, conversion_count,
    is_active, expires_at, created_at, updated_at
FROM campaign_links
WHERE short_code = $1
    AND is_active = TRUE
    AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP);
