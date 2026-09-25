-- Drop independently_metered_tool_charges.
--
-- The table was a forward slot: nothing in core or this repo prices an MCP tool
-- execution (mcp_tool_executions and ai_request_tool_calls carry no cost), so
-- every read of it was a measured-looking zero. Re-introduce it with its
-- producer when a vendor-metered MCP crate exists.
DROP TABLE IF EXISTS independently_metered_tool_charges;
