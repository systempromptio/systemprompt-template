import { useState, useCallback } from 'react'
import { Client } from '@modelcontextprotocol/sdk/client/index.js'
import { StreamableHTTPClientTransport } from '@modelcontextprotocol/sdk/client/streamableHttp.js'
import { useContextStore } from '@/stores/context.store'
import { useAuthStore } from '@/stores/auth.store'
import { useToolExecutionStore } from '@/stores/toolExecution.store'
import { logger } from '@/lib/logger'
import type { EphemeralArtifact, RenderBehavior } from '@/types/artifact'

/**
 * Constructs an ephemeral artifact from tool execution structured content.
 *
 * Extracts artifact type and execution ID from structured content returned
 * by MCP tool execution. Returns null if content is invalid or missing required fields.
 *
 * @param structuredContent - Raw structured content from tool result
 * @param toolName - Name of the tool that produced the artifact
 * @returns EphemeralArtifact object or null if construction fails
 */
function constructEphemeralArtifact(
  structuredContent: unknown,
  toolName: string
): EphemeralArtifact | null {
  if (typeof structuredContent !== 'object' || structuredContent === null) {
    return null
  }

  const data = structuredContent as Record<string, unknown>

  const artifactType = data['x-artifact-type'] as string | undefined
  const executionId = data.mcp_execution_id as string | undefined

  if (!executionId) {
    logger.error('No mcp_execution_id in structured_content', undefined, 'useMcpToolCaller')
    return null
  }

  return {
    artifactId: executionId,
    name: toolName,
    description: `Result from ${toolName}`,
    parts: [
      {
        kind: 'data',
        data: structuredContent as Record<string, unknown>
      }
    ],
    metadata: {
      ephemeral: true,
      artifact_type: artifactType || 'json',
      created_at: new Date().toISOString(),
      source: 'mcp_tool',
      tool_name: toolName,
      mcp_execution_id: executionId
    }
  }
}

/**
 * Hook for calling MCP tools directly from the web frontend.
 *
 * **ARCHITECTURE: ALL ARTIFACTS COME VIA SSE BROADCAST**
 *
 * This hook triggers MCP tool execution but does NOT return artifacts.
 * Instead, artifacts are:
 * 1. Processed and persisted by backend
 * 2. Broadcast via SSE with full metadata
 * 3. Received by frontend stores
 * 4. Used to complete tool execution tracking
 *
 * Flow:
 * 1. Create execution entry in tool execution store
 * 2. Connect to MCP server via HTTP transport with x-context-id header
 * 3. Call tool with arguments (backend persists result)
 * 4. Backend broadcasts task_completed via SSE
 * 5. SSE handler calls completeExecution() to close modal
 *
 * @returns Hook result with callTool function and execution state
 *
 * @throws {Error} When tool execution fails on the server
 * @throws {Error} When MCP connection fails
 *
 * @example
 * ```typescript
 * function ToolExecutor() {
 *   const { callTool, loading, error } = useMcpToolCaller()
 *
 *   const handleExecute = async () => {
 *     try {
 *       await callTool(
 *         'http://localhost:3000/mcp',
 *         'list_files',
 *         { directory: '/tmp' },
 *         'File Server'
 *       )
 *     } catch (err) {
 *       console.error('Tool failed:', err)
 *     }
 *   }
 *
 *   return (
 *     <div>
 *       <button onClick={handleExecute} disabled={loading}>
 *         {loading ? 'Executing...' : 'Run Tool'}
 *       </button>
 *       {error && <p>Error: {error}</p>}
 *     </div>
 *   )
 * }
 * ```
 */
export function useMcpToolCaller() {
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const currentContextId = useContextStore((state) => state.currentContextId)
  const createConversation = useContextStore((state) => state.createConversation)
  const getAuthHeader = useAuthStore((state) => state.getAuthHeader)

  /**
   * Calls an MCP tool on a remote server.
   *
   * Creates a unique execution ID, connects to the MCP server, executes the tool,
   * and processes any ephemeral artifacts returned. Tool execution state is tracked
   * and artifacts are displayed via the artifact store.
   *
   * Parameter values are NEVER logged (only count is logged to avoid PII leakage).
   *
   * @param serverEndpoint - MCP server HTTP endpoint
   * @param toolName - Name of the tool to call
   * @param toolArgs - Tool arguments (parameter keys and values)
   * @param serverName - Optional server display name
   * @param renderBehavior - Optional artifact render mode
   * @throws Error if tool execution fails
   */
  const callTool = useCallback(
    async (
      serverEndpoint: string,
      toolName: string,
      toolArgs: Record<string, unknown>,
      serverName?: string,
      renderBehavior?: RenderBehavior
    ): Promise<void> => {
      logger.debug('Calling tool', {
        tool: toolName,
        paramCount: Object.keys(toolArgs).length
      }, 'useMcpToolCaller')

      const executionId = crypto.randomUUID()

      setLoading(true)
      setError(null)

      const addExecution = useToolExecutionStore.getState().addExecution
      addExecution({
        id: executionId,
        toolName,
        serverName: serverName || 'Unknown',
        status: 'pending',
        timestamp: Date.now(),
        renderBehavior: renderBehavior || 'both',
        parameters: toolArgs
      })

      let contextId = currentContextId
      if (!contextId) {
        try {
          await createConversation('Tool Results')
          contextId = useContextStore.getState().currentContextId
          logger.debug('Created conversation', { contextId }, 'useMcpToolCaller')
        } catch (err) {
          logger.error('Failed to create conversation', err, 'useMcpToolCaller')
        }
      }

      try {
        const authHeader = getAuthHeader()

        const headers: Record<string, string> = {
          'Accept': 'application/json, text/event-stream',
          'x-call-source': 'ephemeral'
        }

        if (authHeader) {
          headers['Authorization'] = authHeader
        }

        const traceId = crypto.randomUUID()
        headers['x-trace-id'] = traceId

        if (contextId) {
          headers['x-context-id'] = contextId
        }

        const apiBaseUrl = import.meta.env.VITE_API_BASE_URL || window.location.origin
        const relativeEndpoint = serverEndpoint.replace(apiBaseUrl, '')

        const transport = new StreamableHTTPClientTransport(
          new URL(relativeEndpoint, window.location.origin),
          {
            requestInit: {
              headers,
            },
          }
        )

        const client = new Client(
          {
            name: 'systemprompt-web-client',
            version: '1.0.0',
          },
          {
            capabilities: {},
          }
        )

        await client.connect(transport)
        logger.debug('Connected to MCP server', { tool: toolName }, 'useMcpToolCaller')

        const result = await client.callTool({
          name: toolName,
          arguments: toolArgs,
        })

        logger.debug('Tool call result', {
          tool: toolName,
          isError: result.isError,
          contentItems: Array.isArray(result.content) ? result.content.length : 0,
          hasStructuredContent: !!result.structuredContent
        }, 'useMcpToolCaller')

        if (result.isError) {
          const content = result.content as Array<{ type: string; text?: string }>
          const errorMessage =
            content.find((c: { type: string; text?: string }) => c.type === 'text')?.text ||
            'Tool execution failed'
          throw new Error(errorMessage)
        }

        await client.close()

        if (result.structuredContent) {
          const ephemeralArtifact = constructEphemeralArtifact(
            result.structuredContent,
            toolName
          )

          if (ephemeralArtifact) {
            const completeExecution = useToolExecutionStore.getState().completeExecution
            completeExecution(executionId, ephemeralArtifact)

            logger.debug('Ephemeral artifact completed', { artifactId: ephemeralArtifact.artifactId }, 'useMcpToolCaller')
          } else {
            logger.warn('Failed to construct artifact from structured_content', undefined, 'useMcpToolCaller')
          }
        }

        setLoading(false)
      } catch (err) {
        let errorMessage = 'Failed to call tool'
        if (err instanceof Error) {
          errorMessage = err.message
        } else if (typeof err === 'string') {
          errorMessage = err
        }

        logger.error('Error calling tool', err, 'useMcpToolCaller')
        setError(errorMessage)
        setLoading(false)

        const failExecution = useToolExecutionStore.getState().failExecution
        failExecution(executionId, errorMessage)

        throw err
      }
    },
    [currentContextId, createConversation, getAuthHeader]
  )

  return {
    callTool,
    loading,
    error,
  }
}
