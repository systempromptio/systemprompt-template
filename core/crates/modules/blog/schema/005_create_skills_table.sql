CREATE TABLE IF NOT EXISTS skills (
    skill_id TEXT PRIMARY KEY,
    skill_name VARCHAR(255) NOT NULL,
    description TEXT,
    content TEXT NOT NULL,
    version VARCHAR(50),
    enabled BOOLEAN DEFAULT true,
    assigned_agents TEXT[],
    allowed_tools TEXT[],
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_skills_enabled ON skills(enabled);
CREATE INDEX IF NOT EXISTS idx_skills_assigned_agents ON skills USING GIN(assigned_agents);
CREATE INDEX IF NOT EXISTS idx_skills_allowed_tools ON skills USING GIN(allowed_tools);

COMMENT ON TABLE skills IS 'Content writing skills loaded dynamically by content-manager tools';
COMMENT ON COLUMN skills.skill_id IS 'Unique identifier for the skill';
COMMENT ON COLUMN skills.skill_name IS 'Human-readable name of the skill';
COMMENT ON COLUMN skills.description IS 'Brief description of what the skill does';
COMMENT ON COLUMN skills.content IS 'Full markdown content of the skill (from index.md)';
COMMENT ON COLUMN skills.version IS 'Semantic version of the skill';
COMMENT ON COLUMN skills.enabled IS 'Whether the skill is currently available';
COMMENT ON COLUMN skills.assigned_agents IS 'Array of agent IDs that can use this skill';
COMMENT ON COLUMN skills.allowed_tools IS 'Array of tool names that can use this skill';
COMMENT ON COLUMN skills.created_at IS 'Timestamp when the skill was created';
COMMENT ON COLUMN skills.updated_at IS 'Timestamp when the skill was last updated';
