-- ============================================================================
-- CONTEXT NOTIFICATIONS - A2A Notification Callback Tracking
-- ============================================================================
-- Tracks notifications received from agents via notification callbacks.
-- Enables event-driven updates for inactive contexts and audit trail.

CREATE TABLE IF NOT EXISTS context_notifications (
    -- Primary key
    id SERIAL PRIMARY KEY,

    -- Context this notification belongs to
    context_id TEXT NOT NULL,

    -- Agent that sent the notification
    agent_id TEXT NOT NULL,

    -- Notification type (A2A method name)
    notification_type TEXT NOT NULL CHECK (
        notification_type IN (
            'notifications/taskStatusUpdate',
            'notifications/artifactCreated',
            'notifications/messageAdded',
            'notifications/contextUpdated'
        )
    ),

    -- Full A2A notification payload (JSON)
    notification_data JSONB NOT NULL,

    -- When notification was received
    received_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    -- Whether notification has been broadcasted to SSE stream
    broadcasted BOOLEAN DEFAULT FALSE,

    -- Foreign key constraints
    FOREIGN KEY (context_id) REFERENCES user_contexts(context_id) ON DELETE CASCADE
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_notifications_context
    ON context_notifications(context_id, received_at DESC);

CREATE INDEX IF NOT EXISTS idx_notifications_not_broadcasted
    ON context_notifications(broadcasted)
    WHERE broadcasted = FALSE;

CREATE INDEX IF NOT EXISTS idx_notifications_agent
    ON context_notifications(agent_id, received_at DESC);

CREATE INDEX IF NOT EXISTS idx_notifications_type
    ON context_notifications(notification_type);
