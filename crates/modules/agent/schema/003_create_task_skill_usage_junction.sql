-- ============================================================================
-- CREATE TASK SKILL USAGE JUNCTION TABLE
-- Tracks which skills were used for which tasks (many-to-many relationship)
-- Enables analytics on skill effectiveness and usage patterns
-- ============================================================================

CREATE TABLE IF NOT EXISTS task_skill_usage (
    id SERIAL PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES agent_tasks(task_id) ON DELETE CASCADE,
    skill_id TEXT NOT NULL,
    usage_type TEXT NOT NULL DEFAULT 'generation',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(task_id, skill_id)
);

CREATE INDEX IF NOT EXISTS idx_task_skill_usage_task_id ON task_skill_usage(task_id);
CREATE INDEX IF NOT EXISTS idx_task_skill_usage_skill_id ON task_skill_usage(skill_id);
CREATE INDEX IF NOT EXISTS idx_task_skill_usage_usage_type ON task_skill_usage(usage_type);
CREATE INDEX IF NOT EXISTS idx_task_skill_usage_created_at ON task_skill_usage(created_at);
