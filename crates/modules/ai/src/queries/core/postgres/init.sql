-- AI Module Core Initialization
-- Creates necessary tables and indexes for the AI module

-- NOTE: ai_requests table moved to schema/ai_requests.sql
-- Load that schema file separately

-- NOTE: tool_executions table removed - was unused
-- For MCP tool execution tracking, use systemprompt_core_mcp::repository::ToolUsageRepository