CREATE TABLE IF NOT EXISTS moltbook_analytics (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL REFERENCES moltbook_agents(id) ON DELETE CASCADE,
    date DATE NOT NULL,
    posts_count INTEGER DEFAULT 0,
    comments_count INTEGER DEFAULT 0,
    upvotes_received INTEGER DEFAULT 0,
    downvotes_received INTEGER DEFAULT 0,
    followers_gained INTEGER DEFAULT 0,
    followers_lost INTEGER DEFAULT 0,
    engagement_score DECIMAL(10, 4) DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_agent_date UNIQUE (agent_id, date)
);

CREATE INDEX idx_moltbook_analytics_agent_id ON moltbook_analytics(agent_id);
CREATE INDEX idx_moltbook_analytics_date ON moltbook_analytics(date DESC);
CREATE INDEX idx_moltbook_analytics_agent_date ON moltbook_analytics(agent_id, date DESC);

CREATE TABLE IF NOT EXISTS moltbook_health_events (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL REFERENCES moltbook_agents(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    message TEXT NOT NULL,
    metadata JSONB,
    resolved BOOLEAN DEFAULT false,
    resolved_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_event_type CHECK (event_type IN (
        'rate_limit_hit', 'downvote_spike', 'report_received',
        'ban_warning', 'prompt_injection_detected', 'api_error'
    )),
    CONSTRAINT valid_severity CHECK (severity IN ('info', 'warning', 'error', 'critical'))
);

CREATE INDEX idx_moltbook_health_events_agent_id ON moltbook_health_events(agent_id);
CREATE INDEX idx_moltbook_health_events_type ON moltbook_health_events(event_type);
CREATE INDEX idx_moltbook_health_events_severity ON moltbook_health_events(severity);
CREATE INDEX idx_moltbook_health_events_unresolved ON moltbook_health_events(agent_id, resolved) WHERE resolved = false;
CREATE INDEX idx_moltbook_health_events_created_at ON moltbook_health_events(created_at DESC);
