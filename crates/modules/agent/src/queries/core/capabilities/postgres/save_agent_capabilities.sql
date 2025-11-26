-- Query: save_agent_capabilities
-- Description: Insert or replace agent capabilities
-- Parameters: uuid, streaming, push_notifications, state_transition_history
-- Returns: Affected rows count

INSERT INTO agent_capabilities (
    uuid,
    streaming,
    push_notifications,
    state_transition_history
) VALUES ($1, $2, $3, $4);