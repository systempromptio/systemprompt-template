WITH candidates AS (
    SELECT r.id, p.request_body_sha256 AS digest,
           analysis_native_session(p.request_body->'metadata'->>'user_id') AS recovered
    FROM ai_requests r
    JOIN ai_request_payloads p ON p.ai_request_id = r.id
    JOIN users u ON u.id = r.user_id
    JOIN user_sessions s ON s.session_id = r.session_id AND s.user_id = r.user_id
    WHERE r.client_session_id IS NULL AND NOT r.synthetic
), recorded AS (
    INSERT INTO ingestion_repairs(request_id, repair_kind, source_digest, recovered_client_session_id)
    SELECT id, 'native_metadata_v1', digest, recovered FROM candidates WHERE recovered IS NOT NULL
    ON CONFLICT(request_id) DO NOTHING RETURNING request_id, recovered_client_session_id
)
UPDATE ai_requests r SET client_session_id = repaired.recovered_client_session_id
FROM recorded repaired WHERE r.id = repaired.request_id AND r.client_session_id IS NULL;
