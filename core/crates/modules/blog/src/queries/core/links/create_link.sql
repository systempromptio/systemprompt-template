INSERT INTO campaign_links (
    id, short_code, target_url, link_type, campaign_id, campaign_name,
    source_content_id, source_page, utm_params, link_text, link_position,
    destination_type, is_active, expires_at, created_at, updated_at
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
RETURNING id;
