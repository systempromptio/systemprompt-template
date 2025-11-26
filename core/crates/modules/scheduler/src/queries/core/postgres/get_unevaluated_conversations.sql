-- Get unique conversations (contexts) that have not been evaluated yet
-- Each context represents one complete conversation with potentially multiple tasks
-- One evaluation per context, aggregating all tasks and messages within that context
SELECT DISTINCT ON (t.context_id)
    t.context_id,
    t.agent_name,
    t.status,
    MIN(t2.started_at) OVER (PARTITION BY t.context_id) as started_at,
    MAX(t2.completed_at) OVER (PARTITION BY t.context_id) as completed_at,
    t.user_id,
    t.session_id,
    t.trace_id
FROM agent_tasks t
JOIN agent_tasks t2 ON t.context_id = t2.context_id
LEFT JOIN conversation_evaluations e ON t.context_id = e.context_id
WHERE e.context_id IS NULL
  AND t.status = 'completed'
  AND EXISTS (
    SELECT 1 FROM task_messages m
    WHERE m.task_id IN (
      SELECT task_id FROM agent_tasks WHERE context_id = t.context_id
    )
  )
ORDER BY t.context_id, t.completed_at DESC
LIMIT $1;
