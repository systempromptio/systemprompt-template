-- Migration: Ensure ai_tool_call_id column exists on ai_request_tool_calls table
-- Purpose: Store the AI provider's tool call identifier (e.g., toolu_01ABC from Anthropic, call_xyz from OpenAI)
--          to enable proper linkage between AI requests and MCP executions
-- Date: 2025-11-21
-- Note: This column is now part of the base schema (ai_request_tool_calls.sql)
--       This migration ensures backwards compatibility for existing databases

-- Add the ai_tool_call_id column if it doesn't exist (idempotent)
ALTER TABLE ai_request_tool_calls
ADD COLUMN IF NOT EXISTS ai_tool_call_id VARCHAR(255);

-- Create index for efficient lookups by provider tool call ID (idempotent)
CREATE INDEX IF NOT EXISTS idx_ai_request_tool_calls_ai_tool_call_id
ON ai_request_tool_calls(ai_tool_call_id)
WHERE ai_tool_call_id IS NOT NULL;

-- Add documentation
COMMENT ON COLUMN ai_request_tool_calls.ai_tool_call_id IS
'AI provider''s tool call identifier from the API response.
Examples:
- Anthropic: toolu_01D7XQ2V9K3J8N5M4P6R7T8W9Y
- OpenAI: call_abc123xyz789
- Gemini: System-generated UUID (provider doesn''t provide IDs)

This field enables bidirectional linkage with mcp_tool_executions.ai_tool_call_id
for complete traceability from AI requests through tool calls to actual executions.

NULL for:
- Providers that don''t provide tool call IDs
- Legacy records before this migration
- Should NOT be NULL for new records from Anthropic/OpenAI';

-- Verify the changes
DO $$
BEGIN
    -- Check column exists
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'ai_request_tool_calls'
        AND column_name = 'ai_tool_call_id'
    ) THEN
        RAISE EXCEPTION 'Migration failed: ai_tool_call_id column was not created';
    END IF;

    -- Check index exists
    IF NOT EXISTS (
        SELECT 1 FROM pg_indexes
        WHERE tablename = 'ai_request_tool_calls'
        AND indexname = 'idx_ai_request_tool_calls_ai_tool_call_id'
    ) THEN
        RAISE EXCEPTION 'Migration failed: index on ai_tool_call_id was not created';
    END IF;

    RAISE NOTICE 'Migration 004 completed successfully: ai_tool_call_id column added';
END $$;
