-- Get new artifacts since last seen timestamp
-- Used by SSE stream to send delta updates for ArtifactCreated events
--
-- Parameters:
-- $1 = user_id
-- $2 = context_id
-- $3 = last_seen_timestamp

SELECT
    ta.artifact_id,
    ta.task_id,
    at.context_id,
    ta.created_at
FROM task_artifacts ta
INNER JOIN agent_tasks at ON ta.task_id = at.task_id
WHERE at.user_id = $1
  AND at.context_id = $2
  AND ta.created_at > $3
ORDER BY ta.created_at ASC;
