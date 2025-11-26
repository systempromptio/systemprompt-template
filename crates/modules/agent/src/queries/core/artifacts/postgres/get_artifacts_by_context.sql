SELECT
    ta.artifact_id,
    ta.name,
    ta.description,
    ta.artifact_type,
    ta.context_id,
    ta.source,
    ta.tool_name,
    ta.mcp_execution_id,
    ta.fingerprint,
    ta.skill_id,
    COALESCE(ags.name, ta.skill_name) AS skill_name,
    ta.metadata,
    ta.task_id,
    ta.created_at AS artifact_created_at
FROM task_artifacts ta
LEFT JOIN agent_skills ags ON ta.skill_id = ags.skill_id
WHERE ta.context_id = $1
ORDER BY ta.created_at DESC
