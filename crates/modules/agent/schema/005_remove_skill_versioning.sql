-- ============================================================================
-- REMOVE SKILL VERSIONING - TECH DEBT CLEANUP
-- Removes all version tracking from skills system
-- ============================================================================

-- Drop old skills table (if exists)
DROP TABLE IF EXISTS skills CASCADE;

-- Remove version columns from agent_skills
ALTER TABLE agent_skills DROP COLUMN IF EXISTS version;
ALTER TABLE agent_skills DROP COLUMN IF EXISTS version_hash;
DROP INDEX IF EXISTS idx_agent_skills_version_hash;

-- Remove skill_version from task_artifacts
ALTER TABLE task_artifacts DROP COLUMN IF EXISTS skill_version;
DROP INDEX IF EXISTS idx_task_artifacts_skill_version;

-- Remove skill_version from task_skill_usage
ALTER TABLE task_skill_usage DROP COLUMN IF EXISTS skill_version;
DROP INDEX IF EXISTS idx_task_skill_usage_skill_version;

-- Recreate analytics view without version
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

COMMENT ON VIEW skill_usage_analytics IS 'Aggregated metrics on skill usage and effectiveness for analytics';
