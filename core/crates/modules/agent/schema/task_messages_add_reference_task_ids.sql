-- Add reference_task_ids column for A2A protocol spec compliance
-- Stores task IDs that this message references for additional context
ALTER TABLE task_messages ADD COLUMN IF NOT EXISTS reference_task_ids TEXT[];
