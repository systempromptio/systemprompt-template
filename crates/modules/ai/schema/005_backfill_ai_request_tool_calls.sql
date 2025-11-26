-- Migration: Backfill ai_request_tool_calls from mcp_tool_executions
-- Purpose: Populate missing ai_request_tool_calls records from MCP executions that have ai_tool_call_id
-- Date: 2025-11-21

-- This migration links MCP executions back to AI requests via ai_tool_call_id
-- and populates ai_request_tool_calls table with the tool call data

BEGIN TRANSACTION;

-- Step 1: Identify MCP executions that have ai_tool_call_id set
-- These represent successful tool calls that should have been recorded in ai_request_tool_calls
-- Use CTEto first determine sequence numbers
WITH to_backfill AS (
    SELECT
        ar.id AS request_id,
        mte.tool_name,
        mte.input AS tool_input,
        mte.mcp_execution_id,
        mte.ai_tool_call_id,
        ROW_NUMBER() OVER (PARTITION BY ar.id ORDER BY mte.created_at ASC) - 1 AS seq_num,
        COALESCE(mte.created_at, CURRENT_TIMESTAMP) AS created_at
    FROM mcp_tool_executions mte
    LEFT JOIN ai_requests ar ON (
        -- Match on user_id, context_id for direct linkage
        ar.user_id = mte.user_id
        AND ar.context_id = mte.context_id
        AND ar.id IS NOT NULL
    )
    WHERE
        -- Only process executions that have ai_tool_call_id (these are the tracked ones)
        mte.ai_tool_call_id IS NOT NULL
        -- Avoid duplicates - only if no tool call record exists for this ai_tool_call_id
        AND NOT EXISTS (
            SELECT 1 FROM ai_request_tool_calls artc
            WHERE artc.ai_tool_call_id = mte.ai_tool_call_id
        )
        -- Must have a matching AI request
        AND ar.id IS NOT NULL
)
INSERT INTO ai_request_tool_calls (
    request_id,
    tool_name,
    tool_input,
    mcp_execution_id,
    ai_tool_call_id,
    sequence_number,
    created_at,
    updated_at
)
SELECT
    request_id,
    tool_name,
    tool_input,
    mcp_execution_id,
    ai_tool_call_id,
    seq_num,
    created_at,
    CURRENT_TIMESTAMP
FROM to_backfill
ON CONFLICT (request_id, sequence_number) DO NOTHING;


-- Step 2: Verify the backfill worked
DO $$
DECLARE
    backfilled_count INTEGER;
    ai_requests_count INTEGER;
    mcp_exec_count INTEGER;
    with_linkage INTEGER;
BEGIN
    SELECT COUNT(*) INTO backfilled_count FROM ai_request_tool_calls;
    SELECT COUNT(*) INTO ai_requests_count FROM ai_requests;
    SELECT COUNT(*) INTO mcp_exec_count FROM mcp_tool_executions WHERE ai_tool_call_id IS NOT NULL;
    SELECT COUNT(*) INTO with_linkage FROM ai_request_tool_calls WHERE mcp_execution_id IS NOT NULL;

    RAISE NOTICE 'Migration 005 completed:';
    RAISE NOTICE '  - AI requests: %', ai_requests_count;
    RAISE NOTICE '  - MCP executions with ai_tool_call_id: %', mcp_exec_count;
    RAISE NOTICE '  - Backfilled ai_request_tool_calls: %', backfilled_count;
    RAISE NOTICE '  - Tool calls with MCP linkage: %', with_linkage;

    IF backfilled_count = 0 THEN
        RAISE NOTICE 'Warning: No records were backfilled. This may indicate no MCP executions have ai_tool_call_id set.';
    END IF;
END $$;

COMMIT;
