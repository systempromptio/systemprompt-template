-- Query: get_agent_capabilities
-- Description: Get agent capabilities by UUID
-- Parameters: uuid
-- Returns: streaming, push_notifications, state_transition_history

SELECT
    streaming,
    push_notifications,
    state_transition_history
FROM agent_capabilities
WHERE uuid = $1;