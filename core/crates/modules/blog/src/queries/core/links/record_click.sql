INSERT INTO link_clicks (
    id, link_id, session_id, user_id, context_id, task_id, referrer_page, referrer_url,
    clicked_at, user_agent, ip_address, device_type, country,
    is_first_click, is_conversion
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
RETURNING id;
