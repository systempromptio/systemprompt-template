-- ============================================================================
-- SIMPLIFY SKILL ID - Remove UUID, use string skill_id as primary key
-- This eliminates confusion between id (UUID) and skill_id (string)
-- ============================================================================

-- First, update any task_artifacts that reference the old UUID id
-- to use the string skill_id instead
UPDATE task_artifacts ta
SET skill_id = (
    SELECT skill_id FROM agent_skills WHERE id = ta.skill_id
)
WHERE ta.skill_id IN (SELECT id FROM agent_skills WHERE id != skill_id);

-- Drop the idx_agent_skills_skill_id index (will be recreated as PK)
DROP INDEX IF EXISTS idx_agent_skills_skill_id;

-- Truncate agent_skills table - skills will be re-ingested on startup
TRUNCATE TABLE agent_skills CASCADE;

-- Drop and recreate agent_skills with skill_id as primary key
DROP TABLE IF EXISTS agent_skills CASCADE;

CREATE TABLE agent_skills (
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

-- Recreate the analytics view with the new schema
DROP VIEW IF EXISTS skill_usage_analytics;
CREATE OR REPLACE VIEW skill_usage_analytics AS
SELECT
    s.skill_id,
    s.name AS skill_name,
    s.description AS skill_description,
    s.category_id,
    COUNT(DISTINCT a.task_id) AS tasks_using_skill,
    COUNT(a.artifact_id) AS artifacts_created,
    COUNT(CASE WHEN t.status = 'completed' THEN 1 END) AS successful_tasks,
    COUNT(CASE WHEN t.status = 'failed' THEN 1 END) AS failed_tasks,
    MIN(a.created_at) AS first_used_at,
    MAX(a.created_at) AS last_used_at
FROM agent_skills s
LEFT JOIN task_artifacts a ON a.skill_id = s.skill_id
LEFT JOIN agent_tasks t ON t.task_id = a.task_id
GROUP BY s.skill_id, s.name, s.description, s.category_id;

COMMENT ON VIEW skill_usage_analytics IS 'Aggregated metrics on skill usage and effectiveness';
