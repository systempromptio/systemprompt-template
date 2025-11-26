-- ============================================================================
-- CONTEXT AGENTS - Multi-Agent Context Tracking
-- ============================================================================
-- Tracks which agents are participating in which contexts.
-- Enables multi-agent conversations where multiple agents can contribute
-- to a single context/conversation.

CREATE TABLE IF NOT EXISTS context_agents (
    id SERIAL PRIMARY KEY,

    -- Context this agent is part of
    context_id TEXT NOT NULL,

    -- Agent identifier (name from services.yaml)
    agent_name TEXT NOT NULL,

    -- When the agent was added to this context
    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    -- Last time this agent was active in this context
    last_active_at TIMESTAMP,

    -- Foreign key constraints
    FOREIGN KEY (context_id) REFERENCES user_contexts(context_id) ON DELETE CASCADE,

    -- Unique constraint: one agent per context
    UNIQUE(context_id, agent_name)
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_context_agents_context
    ON context_agents(context_id);

CREATE INDEX IF NOT EXISTS idx_context_agents_agent_name
    ON context_agents(agent_name);

CREATE INDEX IF NOT EXISTS idx_context_agents_active
    ON context_agents(context_id, last_active_at DESC);
