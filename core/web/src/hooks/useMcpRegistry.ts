import { useEffect, useRef } from 'react'
import { Client } from '@modelcontextprotocol/sdk/client/index.js'
import { StreamableHTTPClientTransport } from '@modelcontextprotocol/sdk/client/streamableHttp.js'
import { useToolsStore, type McpTool, type McpServer } from '@/stores/tools.store'
import { useAuthStore } from '@/stores/auth.store'
import { useContextStore } from '@/stores/context.store'
import { logger } from '@/lib/logger'

/**
 * Evaluates permission hierarchy based on authorization scopes.
 *
 * Implements a scope hierarchy where higher scopes grant all permissions
 * of lower scopes (e.g., admin > user > service > a2a > mcp > anonymous).
 *
 * @param userScope - User's current authorization scope
 * @param requiredScope - Scope required to access a resource
 * @returns true if user's scope satisfies required scope
 */
function permissionImplies(userScope: string, requiredScope: string): boolean {
  const levels: Record<string, number> = {
    'admin': 100,
    'user': 50,
    'service': 40,
    'a2a': 30,
    'mcp': 20,
    'anonymous': 10,
  }
  const userLevel = levels[userScope.toLowerCase()] || 0
  const requiredLevel = levels[requiredScope.toLowerCase()] || 0
  return userLevel >= requiredLevel
}

interface McpRegistryServer {
  name: string
  version: string
  description: string
  port: number
  enabled: boolean
  oauth_required: boolean
  oauth_scopes: string[]
  endpoint: string
  status: string
}

/**
 * Loads and manages Model Context Protocol (MCP) servers and their tools.
 *
 * Fetches the MCP registry, connects to each enabled server, and collects
 * all available tools. Checks user permissions against server OAuth requirements.
 * Tools are deduped by server + tool name and stored in the tools store.
 *
 * Auto-runs on mount and performs initial setup. Prevents duplicate execution
 * in React StrictMode using an internal flag.
 *
 * @returns void - Registry state is managed via tools store
 *
 * @throws {Error} When MCP registry API request fails
 * @throws {Error} When MCP server connection fails
 * @throws {Error} When JSON parsing of server response fails
 * @throws {Error} When tool list retrieval fails
 *
 * @example
 * ```typescript
 * useMcpRegistry() // Auto-runs in app root
 * const { tools, servers, loading } = useToolsStore()
 * ```
 */
export function useMcpRegistry() {
  const { setTools, setServers, setLoading, setError, clearTools } = useToolsStore()
  const getAuthHeader = useAuthStore((state) => state.getAuthHeader)
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated)
  const userScopes = useAuthStore((state) => state.scopes)
  const currentContextId = useContextStore((state) => state.currentContextId)
  const isLoadingRef = useRef(false)

  useEffect(() => {
    const loadMcpRegistry = async () => {
      if (isLoadingRef.current) {
        logger.debug('Registry load already in progress, skipping', undefined, 'useMcpRegistry')
        return
      }

      logger.debug('Starting to load MCP registry', undefined, 'useMcpRegistry')

      isLoadingRef.current = true
      clearTools()
      setLoading(true)
      setError(null)

      try {
        const authHeader = getAuthHeader()
        if (!authHeader) {
          logger.error('Missing authentication', new Error('No JWT token available'), 'useMcpRegistry')
          throw new Error('Missing authentication')
        }

        const registryResponse = await fetch('/api/v1/mcp/registry', {
          headers: {
            'Authorization': authHeader,
          },
        })

        if (!registryResponse.ok) {
          logger.error('Registry API failed', new Error(`${registryResponse.statusText}`), 'useMcpRegistry')
          throw new Error(`Registry API failed: ${registryResponse.statusText}`)
        }

        const registryData = await registryResponse.json()

        const registryServers: McpRegistryServer[] = registryData.data || []
        logger.debug('Found registry servers', { count: registryServers.length }, 'useMcpRegistry')

        const apiBaseUrl = import.meta.env.VITE_API_BASE_HOST || window.location.origin
        const servers: McpServer[] = registryServers
          .filter(s => s.enabled)
          .map(s => ({
            name: s.name,
            endpoint: s.endpoint.replace(apiBaseUrl, ''),
            auth: s.oauth_required ? 'required' : 'none',
            status: s.status,
            oauth_required: s.oauth_required,
            oauth_scopes: s.oauth_scopes
          }))

        logger.debug('Filtered enabled servers', { count: servers.length }, 'useMcpRegistry')

        setServers(servers)

        const allTools: McpTool[] = []

        for (const server of servers) {
          logger.debug('Connecting to server', { name: server.name }, 'useMcpRegistry')

          if (server.oauth_required) {
            const requiredScopes = server.oauth_scopes || []
            const hasPermission = requiredScopes.length === 0 || requiredScopes.some((requiredScope) =>
              userScopes.some((userScope) => permissionImplies(userScope, requiredScope))
            )

            if (!hasPermission) {
              logger.debug('Skipping server due to insufficient permissions', { name: server.name }, 'useMcpRegistry')
              continue
            }
          }

          try {
            const authHeader = getAuthHeader()

            const headers: Record<string, string> = {
              'Accept': 'application/json, text/event-stream'
            }

            if (server.auth === 'required' && authHeader) {
              headers['Authorization'] = authHeader
            }

            const traceId = crypto.randomUUID()
            headers['X-Trace-ID'] = traceId

            if (currentContextId) {
              headers['X-Context-ID'] = currentContextId
            }

            const transport = new StreamableHTTPClientTransport(
              new URL(server.endpoint, window.location.origin),
              {
                requestInit: {
                  headers
                }
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
            logger.debug('Connected to server', { name: server.name }, 'useMcpRegistry')

            const result = await client.listTools()
            logger.debug('Received tools from server', { name: server.name, count: result.tools.length }, 'useMcpRegistry')

            const tools: McpTool[] = result.tools.map((tool) => ({
              ...tool,
              serverName: server.name,
              serverEndpoint: server.endpoint,
            }))

            allTools.push(...tools)

            await client.close()
          } catch (error) {
            logger.error(`Failed to connect to MCP server`, error, 'useMcpRegistry')
          }
        }

        logger.debug('Total tools loaded', { count: allTools.length }, 'useMcpRegistry')
        setTools(allTools)

        // If no servers found, set helpful error message
        if (servers.length === 0) {
          setError('No MCP servers running. Start MCP servers to see tools.')
        }
      } catch (error) {
        logger.error('Error loading MCP registry', error, 'useMcpRegistry')
        setError(error instanceof Error ? error.message : 'Failed to load MCP registry')
      } finally {
        isLoadingRef.current = false
        setLoading(false)
      }
    }

    loadMcpRegistry()
  }, [setTools, setServers, setLoading, setError, clearTools, getAuthHeader, isAuthenticated, userScopes, currentContextId])
}
