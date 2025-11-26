-- Log table for system-wide logging with analytics support
CREATE TABLE IF NOT EXISTS logs (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    level VARCHAR(50) NOT NULL CHECK (level IN ('ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE')),
    module VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    metadata TEXT, -- JSON blob for additional structured data
    -- Analytics Context
    user_id VARCHAR(255),
    session_id VARCHAR(255),
    task_id VARCHAR(255),
    trace_id VARCHAR(255),
    context_id VARCHAR(255),
    client_id VARCHAR(255),
    CONSTRAINT log_level_check CHECK (level IN ('ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE'))
);
-- Existing indexes
CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON logs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_logs_level ON logs(level);
CREATE INDEX IF NOT EXISTS idx_logs_module ON logs(module);
CREATE INDEX IF NOT EXISTS idx_logs_level_timestamp ON logs(level, timestamp DESC);
-- Analytics indexes
CREATE INDEX IF NOT EXISTS idx_logs_user_id ON logs(user_id);
CREATE INDEX IF NOT EXISTS idx_logs_session_id ON logs(session_id);
CREATE INDEX IF NOT EXISTS idx_logs_task_id ON logs(task_id);
CREATE INDEX IF NOT EXISTS idx_logs_trace_id ON logs(trace_id);
CREATE INDEX IF NOT EXISTS idx_logs_context_id ON logs(context_id);
CREATE INDEX IF NOT EXISTS idx_logs_user_timestamp ON logs(user_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_logs_session_timestamp ON logs(session_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_logs_context_timestamp ON logs(context_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_logs_client_id ON logs(client_id);
CREATE INDEX IF NOT EXISTS idx_logs_client_timestamp ON logs(client_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_logs_client_level ON logs(client_id, level);
DROP VIEW IF EXISTS v_log_analytics_by_client CASCADE;
CREATE VIEW v_log_analytics_by_client AS
SELECT
    client_id,
    level,
    module,
    COUNT(*) as log_count,
    MIN(timestamp) as first_seen,
    MAX(timestamp) as last_seen
FROM logs
WHERE client_id IS NOT NULL
GROUP BY client_id, level, module
ORDER BY log_count DESC;
DROP VIEW IF EXISTS v_client_errors CASCADE;
CREATE VIEW v_client_errors AS
SELECT
    client_id,
    COUNT(*) as error_count,
    COUNT(DISTINCT session_id) as affected_sessions,
    MAX(timestamp) as last_error
FROM logs
WHERE level = 'ERROR' AND client_id IS NOT NULL
GROUP BY client_id
ORDER BY error_count DESC;