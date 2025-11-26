-- ============================================================================
-- ADD SKILL TRACKING TO TASK ARTIFACTS
-- Adds skill metadata columns to enable skill attribution and analytics
-- ============================================================================

ALTER TABLE task_artifacts
ADD COLUMN IF NOT EXISTS skill_id TEXT,
ADD COLUMN IF NOT EXISTS skill_name TEXT;

CREATE INDEX IF NOT EXISTS idx_task_artifacts_skill_id ON task_artifacts(skill_id);
CREATE INDEX IF NOT EXISTS idx_task_artifacts_skill_name ON task_artifacts(skill_name);
