CREATE TABLE IF NOT EXISTS moltbook_agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    api_key_hash TEXT,
    persona TEXT NOT NULL,
    submolts JSONB DEFAULT '[]'::jsonb,
    cadence_posts_per_week INTEGER DEFAULT 0,
    cadence_comments_per_day INTEGER DEFAULT 0,
    enabled BOOLEAN DEFAULT false,
    verified BOOLEAN DEFAULT false,
    followers_count INTEGER DEFAULT 0,
    following_count INTEGER DEFAULT 0,
    posts_count INTEGER DEFAULT 0,
    comments_count INTEGER DEFAULT 0,
    last_sync_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    CONSTRAINT name_not_empty CHECK (name != ''),
    CONSTRAINT description_not_empty CHECK (description != '')
);

CREATE INDEX idx_moltbook_agents_name ON moltbook_agents(name);
CREATE INDEX idx_moltbook_agents_enabled ON moltbook_agents(enabled) WHERE enabled = true;
CREATE INDEX idx_moltbook_agents_persona ON moltbook_agents(persona);
