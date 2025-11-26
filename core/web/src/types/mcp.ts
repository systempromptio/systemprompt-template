export interface ToolExecutionResponse {
  id: string
  tool_name: string
  mcp_server_name: string
  server_endpoint: string
  input: Record<string, unknown>
  output: Record<string, unknown> | null
  status: string
}
