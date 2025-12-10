CREATE TABLE IF NOT EXISTS task_execution_steps (
    id SERIAL PRIMARY KEY,
    step_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    step_type TEXT NOT NULL,
    step_number INTEGER NOT NULL,
    iteration_number INTEGER,
    title TEXT NOT NULL,
    subtitle TEXT,
    reasoning TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    tool_name TEXT,
    tool_arguments JSONB,
    tool_result JSONB,
    error_message TEXT,
    execution_mode TEXT,
    estimated_total_steps INTEGER,
    progress_percentage INTEGER,
    decision_type TEXT,
    synthesized_response TEXT,
    next_tool TEXT,
    next_tool_args JSONB,
    started_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMPTZ,
    duration_ms INTEGER,
    user_id TEXT,
    session_id TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_task_execution_steps_task_id ON task_execution_steps(task_id);
CREATE INDEX IF NOT EXISTS idx_task_execution_steps_step_id ON task_execution_steps(step_id);
