-- MCP Tool Executions - Comprehensive tracking of MCP tool calls
-- Created: 2025-10-01
-- Purpose: Track all MCP tool invocations with full analytics context
CREATE TABLE IF NOT EXISTS mcp_tool_executions (
    mcp_execution_id TEXT PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Tool identification
    tool_name VARCHAR(255) NOT NULL,
    mcp_server_name VARCHAR(255) NOT NULL,
    -- Execution timing
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    execution_time_ms INTEGER,
    -- Execution data
    input TEXT NOT NULL,
    output TEXT,
    output_schema TEXT,
    status VARCHAR(255) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'success', 'failed', 'timeout')),
    error_message TEXT,
    -- Full analytics context (user_id required for attribution)
    user_id VARCHAR(255) NOT NULL,
    session_id VARCHAR(255),
    context_id VARCHAR(255),
    task_id VARCHAR(255),
    trace_id VARCHAR(255),
    -- Request metadata
    request_method TEXT,
    request_source TEXT,
    ai_tool_call_id VARCHAR(255),
    -- Audit
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_tool_name ON mcp_tool_executions(tool_name);
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_server_name ON mcp_tool_executions(mcp_server_name);
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_status ON mcp_tool_executions(status);
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_started_at ON mcp_tool_executions(started_at DESC);
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_completed_at ON mcp_tool_executions(completed_at DESC);
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_execution_time ON mcp_tool_executions(execution_time_ms DESC);
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_created_at ON mcp_tool_executions(created_at DESC);
-- Analytics indexes
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_user_id ON mcp_tool_executions(user_id);
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_session_id ON mcp_tool_executions(session_id);
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_context_id ON mcp_tool_executions(context_id);
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_task_id ON mcp_tool_executions(task_id);
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_trace_id ON mcp_tool_executions(trace_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_mcp_tool_executions_ai_tool_call_id ON mcp_tool_executions(ai_tool_call_id) WHERE ai_tool_call_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_mcp_execution_id ON mcp_tool_executions(mcp_execution_id);
-- Compound indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_user_created ON mcp_tool_executions(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_context_created ON mcp_tool_executions(context_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_session_tool ON mcp_tool_executions(session_id, tool_name);
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_tool_status ON mcp_tool_executions(tool_name, status);
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_server_tool ON mcp_tool_executions(mcp_server_name, tool_name);
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_server_status ON mcp_tool_executions(mcp_server_name, status);
CREATE INDEX IF NOT EXISTS idx_mcp_tool_executions_tool_started ON mcp_tool_executions(tool_name, started_at DESC);
