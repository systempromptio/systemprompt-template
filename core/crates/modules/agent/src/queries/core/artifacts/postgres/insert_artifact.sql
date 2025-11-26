INSERT INTO task_artifacts (
    task_id,
    context_id,
    artifact_id,
    name,
    description,
    artifact_type,
    source,
    tool_name,
    mcp_execution_id,
    fingerprint,
    skill_id,
    skill_name,
    metadata
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13::jsonb)
ON CONFLICT (task_id, artifact_id) DO NOTHING
