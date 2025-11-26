-- ============================================================================
-- TASK PUSH NOTIFICATION CONFIGS - A2A Spec-Compliant Multi-Endpoint Support
-- ============================================================================
-- Stores multiple push notification endpoints per task as per A2A spec.
-- Supports tasks/pushNotificationConfig/* methods.

CREATE TABLE IF NOT EXISTS task_push_notification_configs (
    -- Primary key
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,

    -- Task this notification config belongs to
    task_id TEXT NOT NULL,

    -- Notification endpoint URL (A2A spec field)
    url TEXT NOT NULL,

    -- Endpoint identifier (A2A spec field)
    endpoint TEXT NOT NULL,

    -- Optional Bearer token for authentication
    token TEXT,

    -- Optional custom headers (JSON)
    headers JSONB,

    -- Optional authentication config (JSON)
    authentication JSONB,

    -- Metadata
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    -- Foreign key constraints
    FOREIGN KEY (task_id) REFERENCES agent_tasks(task_id) ON DELETE CASCADE,

    -- Unique constraint: one config per (task_id, endpoint) pair
    UNIQUE(task_id, endpoint)
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_task_push_configs_task_id
    ON task_push_notification_configs(task_id);

CREATE INDEX IF NOT EXISTS idx_task_push_configs_created
    ON task_push_notification_configs(created_at DESC);

-- Unique constraint: one config per (task_id, endpoint) pair
CREATE UNIQUE INDEX IF NOT EXISTS idx_task_push_configs_task_endpoint
    ON task_push_notification_configs(task_id, endpoint);
