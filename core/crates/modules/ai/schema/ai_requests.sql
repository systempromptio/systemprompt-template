-- ============================================================================
-- AI REQUESTS - Track all AI provider interactions with user context
-- ============================================================================
--
-- DATA RETENTION POLICY:
-- - Production: Retain completed requests for 90 days
-- - Failed requests: Retain for 30 days for debugging
-- - Pending requests older than 24 hours should be investigated
-- - Cost and usage data: Aggregate monthly before deletion
-- - Implement automatic cleanup via scheduled job or trigger
--
-- QUERY PATTERNS:
-- - User analytics: (user_id, created_at) - time-series user metrics
-- - Model analytics: (user_id, model) - per-model usage tracking
-- - Provider health: (provider, status) - service reliability monitoring
-- - Session tracking: (session_id, created_at) - conversation flow analysis
-- ============================================================================
CREATE TABLE IF NOT EXISTS ai_requests (
    id TEXT PRIMARY KEY,
    request_id VARCHAR(255) NOT NULL UNIQUE,
    -- Analytics Context
    user_id VARCHAR(255) NOT NULL,
    session_id VARCHAR(255),
    task_id VARCHAR(255),
    context_id VARCHAR(255),
    trace_id VARCHAR(255),
    -- Request Details
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    -- Sampling Parameters (promoted from sampling_metadata JSON)
    temperature DOUBLE PRECISION,
    top_p DOUBLE PRECISION,
    max_tokens INTEGER,
    stop_sequences TEXT,  -- JSON array (low cardinality)
    -- Usage Metrics
    tokens_used INTEGER,
    input_tokens INTEGER,
    output_tokens INTEGER,
    cost_cents INTEGER DEFAULT 0,
    latency_ms INTEGER,
    -- Cache Tracking
    cache_hit BOOLEAN DEFAULT FALSE,
    cache_read_tokens INTEGER,
    cache_creation_tokens INTEGER,
    -- Request Type
    is_streaming BOOLEAN DEFAULT FALSE,
    -- Status Tracking
    status VARCHAR(255) NOT NULL DEFAULT 'pending',
    error_message TEXT,
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP
);
-- Performance Indexes
CREATE INDEX IF NOT EXISTS idx_ai_requests_request_id ON ai_requests(request_id);
CREATE INDEX IF NOT EXISTS idx_ai_requests_provider ON ai_requests(provider);
CREATE INDEX IF NOT EXISTS idx_ai_requests_status ON ai_requests(status);
CREATE INDEX IF NOT EXISTS idx_ai_requests_created_at ON ai_requests(created_at);
-- Analytics Indexes (user context)
CREATE INDEX IF NOT EXISTS idx_ai_requests_user_id ON ai_requests(user_id);
CREATE INDEX IF NOT EXISTS idx_ai_requests_session_id ON ai_requests(session_id);
CREATE INDEX IF NOT EXISTS idx_ai_requests_task_id ON ai_requests(task_id);
CREATE INDEX IF NOT EXISTS idx_ai_requests_context_id ON ai_requests(context_id);
CREATE INDEX IF NOT EXISTS idx_ai_requests_trace_id ON ai_requests(trace_id);
CREATE INDEX IF NOT EXISTS idx_ai_requests_cost ON ai_requests(cost_cents);

-- Composite Indexes (query optimization)
CREATE INDEX IF NOT EXISTS idx_ai_requests_user_created ON ai_requests(user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_ai_requests_user_model ON ai_requests(user_id, model);
CREATE INDEX IF NOT EXISTS idx_ai_requests_provider_status ON ai_requests(provider, status);
CREATE INDEX IF NOT EXISTS idx_ai_requests_session_created ON ai_requests(session_id, created_at);