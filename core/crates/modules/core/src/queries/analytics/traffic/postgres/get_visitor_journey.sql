WITH visitor_stats AS (
    SELECT
        COUNT(DISTINCT fingerprint_hash) as total_visitors,
        COUNT(DISTINCT CASE
            WHEN request_count = 1 THEN session_id
        END) * 100.0 / NULLIF(COUNT(*), 0) as bounce_rate,
        AVG(request_count) as avg_pages_per_session
    FROM user_sessions
    WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
      AND is_bot = false
      AND is_scanner = false
      AND request_count > 0
),
new_vs_returning AS (
    SELECT
        COUNT(DISTINCT CASE
            WHEN first_ever_session >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL THEN fingerprint_hash
        END) as new_visitors,
        COUNT(DISTINCT CASE
            WHEN first_ever_session < CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL THEN fingerprint_hash
        END) as returning_visitors
    FROM (
        SELECT DISTINCT ON (us.fingerprint_hash)
            us.fingerprint_hash,
            (SELECT MIN(started_at)
             FROM user_sessions us_inner
             WHERE us_inner.fingerprint_hash = us.fingerprint_hash
               AND us_inner.is_bot = false
               AND us_inner.is_scanner = false
            ) as first_ever_session
        FROM user_sessions us
        WHERE us.started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
          AND us.is_bot = false
          AND us.is_scanner = false
          AND us.request_count > 0
    ) sub
)
SELECT
    vs.total_visitors,
    nvr.new_visitors,
    nvr.returning_visitors,
    vs.avg_pages_per_session,
    vs.bounce_rate,
    (nvr.returning_visitors::float / NULLIF(vs.total_visitors, 0) * 100) as return_visitor_rate
FROM visitor_stats vs, new_vs_returning nvr
