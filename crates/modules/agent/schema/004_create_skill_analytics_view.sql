-- ============================================================================
-- CREATE SKILL ANALYTICS VIEW
-- Provides aggregated metrics on skill usage and effectiveness
-- ============================================================================

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
    COUNT(CASE WHEN t.status IN ('running', 'pending') THEN 1 END) AS in_progress_tasks,
    MIN(a.created_at) AS first_used_at,
    MAX(a.created_at) AS last_used_at
FROM agent_skills s
LEFT JOIN task_artifacts a ON a.skill_id = s.skill_id
LEFT JOIN agent_tasks t ON t.task_id = a.task_id
GROUP BY s.skill_id, s.name, s.description, s.category_id;

COMMENT ON VIEW skill_usage_analytics IS 'Aggregated metrics on skill usage and effectiveness for analytics';
