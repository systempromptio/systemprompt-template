-- ============================================================================
-- ENDPOINT REQUESTS - Track individual HTTP endpoint requests per session
-- ============================================================================

CREATE TABLE IF NOT EXISTS endpoint_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id VARCHAR(255) NOT NULL,
    endpoint_path TEXT NOT NULL,
    http_method VARCHAR(10) NOT NULL,
    response_status INTEGER,
    response_time_ms INTEGER,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (session_id) REFERENCES user_sessions(session_id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_endpoint_requests_session ON endpoint_requests(session_id);
CREATE INDEX IF NOT EXISTS idx_endpoint_requests_endpoint ON endpoint_requests(endpoint_path);
CREATE INDEX IF NOT EXISTS idx_endpoint_requests_requested_at ON endpoint_requests(requested_at);
CREATE INDEX IF NOT EXISTS idx_endpoint_requests_composite ON endpoint_requests(endpoint_path, http_method, requested_at);
CREATE INDEX IF NOT EXISTS idx_endpoint_requests_status ON endpoint_requests(response_status);
CREATE INDEX IF NOT EXISTS idx_endpoint_requests_response_time ON endpoint_requests(response_time_ms);
