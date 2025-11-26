CREATE TABLE IF NOT EXISTS agent_skills (
    skill_id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL UNIQUE,

    name TEXT NOT NULL,
    description TEXT NOT NULL,
    instructions TEXT NOT NULL,

    enabled BOOLEAN NOT NULL DEFAULT true,

    allowed_tools TEXT[],
    tags TEXT[],

    category_id TEXT,
    source_id TEXT NOT NULL,

    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_agent_skills_enabled ON agent_skills(enabled);
CREATE INDEX IF NOT EXISTS idx_agent_skills_source ON agent_skills(source_id);
CREATE INDEX IF NOT EXISTS idx_agent_skills_category ON agent_skills(category_id);
