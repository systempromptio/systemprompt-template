-- Simplify task_execution_steps to use JSONB content column
-- This removes the denormalized columns and uses a single content JSONB field

-- Add the content column
ALTER TABLE task_execution_steps ADD COLUMN IF NOT EXISTS content JSONB;

-- Migrate existing data to the new content column format
UPDATE task_execution_steps SET content =
  CASE step_type
    WHEN 'understanding' THEN '{"type": "understanding"}'::jsonb
    WHEN 'planning' THEN jsonb_build_object('type', 'planning', 'reasoning', reasoning)
    WHEN 'skill_usage' THEN jsonb_build_object('type', 'skill_usage', 'skill_id', COALESCE(tool_name, ''), 'skill_name', COALESCE(tool_name, ''))
    WHEN 'tool_execution' THEN jsonb_build_object(
      'type', 'tool_execution',
      'tool_name', COALESCE(tool_name, ''),
      'tool_arguments', COALESCE(tool_arguments, '{}'::jsonb),
      'tool_result', tool_result
    )
    WHEN 'completion' THEN '{"type": "completion"}'::jsonb
    ELSE '{"type": "understanding"}'::jsonb
  END
WHERE content IS NULL;

-- Make content NOT NULL after migration
ALTER TABLE task_execution_steps ALTER COLUMN content SET NOT NULL;

-- Drop the old columns that are now in content
ALTER TABLE task_execution_steps DROP COLUMN IF EXISTS step_type;
ALTER TABLE task_execution_steps DROP COLUMN IF EXISTS step_number;
ALTER TABLE task_execution_steps DROP COLUMN IF EXISTS iteration_number;
ALTER TABLE task_execution_steps DROP COLUMN IF EXISTS title;
ALTER TABLE task_execution_steps DROP COLUMN IF EXISTS subtitle;
ALTER TABLE task_execution_steps DROP COLUMN IF EXISTS reasoning;
ALTER TABLE task_execution_steps DROP COLUMN IF EXISTS tool_name;
ALTER TABLE task_execution_steps DROP COLUMN IF EXISTS tool_arguments;
ALTER TABLE task_execution_steps DROP COLUMN IF EXISTS tool_result;
ALTER TABLE task_execution_steps DROP COLUMN IF EXISTS execution_mode;
ALTER TABLE task_execution_steps DROP COLUMN IF EXISTS estimated_total_steps;
ALTER TABLE task_execution_steps DROP COLUMN IF EXISTS progress_percentage;
ALTER TABLE task_execution_steps DROP COLUMN IF EXISTS decision_type;
ALTER TABLE task_execution_steps DROP COLUMN IF EXISTS synthesized_response;
ALTER TABLE task_execution_steps DROP COLUMN IF EXISTS next_tool;
ALTER TABLE task_execution_steps DROP COLUMN IF EXISTS next_tool_args;
ALTER TABLE task_execution_steps DROP COLUMN IF EXISTS user_id;
ALTER TABLE task_execution_steps DROP COLUMN IF EXISTS session_id;

-- Create index on content for type queries
CREATE INDEX IF NOT EXISTS idx_task_execution_steps_content_type
ON task_execution_steps ((content->>'type'));
