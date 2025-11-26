INSERT INTO user_sessions (
    session_id,
    user_id,
    user_type,
    expires_at,
    fingerprint_hash,
    ip_address,
    user_agent,
    device_type,
    browser,
    os,
    country,
    region,
    city,
    preferred_locale,
    referrer_source,
    referrer_url,
    landing_page,
    entry_url,
    utm_source,
    utm_medium,
    utm_campaign,
    is_bot
)
VALUES (
    $1,
    $2,
    CASE WHEN $2 IS NULL THEN 'anon' ELSE 'registered' END,
    $21,
    $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20
)