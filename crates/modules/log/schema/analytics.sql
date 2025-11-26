CREATE TABLE IF NOT EXISTS analytics_events (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id VARCHAR(255) NOT NULL,
    session_id TEXT,
    context_id VARCHAR(255),
    event_type VARCHAR(255) NOT NULL,
    event_category TEXT NOT NULL,
    severity TEXT NOT NULL,
    endpoint TEXT,
    error_code INTEGER,
    response_time_ms INTEGER,
    agent_id VARCHAR(255),
    task_id VARCHAR(255),
    message TEXT,
    metadata TEXT, -- Stored as a JSON string
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Foreign key for session_id with SET NULL (sessions are ephemeral metadata)
    CONSTRAINT analytics_events_session_id_fkey
        FOREIGN KEY (session_id)
        REFERENCES user_sessions(session_id)
        ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_analytics_events_user_id ON analytics_events(user_id);
CREATE INDEX IF NOT EXISTS idx_analytics_events_session_id ON analytics_events(session_id);
CREATE INDEX IF NOT EXISTS idx_analytics_events_context_id ON analytics_events(context_id);
CREATE INDEX IF NOT EXISTS idx_analytics_events_event_type ON analytics_events(event_type);
CREATE INDEX IF NOT EXISTS idx_analytics_events_event_category ON analytics_events(event_category);
CREATE INDEX IF NOT EXISTS idx_analytics_events_timestamp ON analytics_events(timestamp);
