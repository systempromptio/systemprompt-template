import { create } from 'zustand'
import type { Tool } from '@modelcontextprotocol/sdk/types.js'

/**
 * Extended MCP Tool with server metadata
 *
 * Extends the official MCP SDK Tool type with application-specific
 * fields for tracking which server the tool comes from.
 *
 * Note: Tool already includes inputSchema and outputSchema from the MCP spec.
 */
export interface McpTool extends Tool {
  serverName: string
  serverEndpoint: string
}

/**
 * Represents an MCP server configuration and status
 */
export interface McpServer {
  name: string
  endpoint: string
  auth: string
  status: string
  oauth_required?: boolean
  oauth_scopes?: string[]
}

/**
 * Zustand store interface for managing MCP tools and servers
 */
interface ToolsStore {
  tools: readonly McpTool[]
  servers: readonly McpServer[]
  loading: boolean
  error: string | null

  setTools: (tools: McpTool[]) => void
  setServers: (servers: McpServer[]) => void
  setLoading: (loading: boolean) => void
  setError: (error: string | null) => void
  clearTools: () => void
}

/**
 * Zustand store for managing MCP (Model Context Protocol) tools and servers
 *
 * State is organized as:
 * - tools: Array of all available MCP tools from connected servers
 * - servers: Array of all registered MCP servers and their status
 * - loading: Loading state indicator
 * - error: Error message if any
 *
 * This store provides a centralized registry of MCP tools and their
 * associated servers, enabling tool discovery and execution
 */
export const useToolsStore = create<ToolsStore>((set) => ({
  tools: [],
  servers: [],
  loading: false,
  error: null,

  /**
   * Sets the complete list of available tools
   * Replaces all existing tools with the new list
   *
   * @param tools - Array of MCP tools to set
   */
  setTools: (tools) => set({ tools }),

  /**
   * Sets the complete list of registered servers
   * Replaces all existing servers with the new list
   *
   * @param servers - Array of MCP servers to set
   */
  setServers: (servers) => set({ servers }),

  /**
   * Sets the loading state
   *
   * @param loading - True if loading, false otherwise
   */
  setLoading: (loading) => set({ loading }),

  /**
   * Sets the error state
   *
   * @param error - Error message or null to clear error
   */
  setError: (error) => set({ error }),

  /**
   * Clears all tools and servers from the store
   * Resets to initial empty state and clears any error
   */
  clearTools: () => set({ tools: [], servers: [], error: null }),
}))

/**
 * Selector functions for accessing tools store state
 */
export const toolsSelectors = {
  /**
   * Gets a tool by its name
   *
   * @param state - Tools store state
   * @param name - Tool name to look up
   * @returns McpTool if found, undefined otherwise
   */
  getToolByName: (state: ToolsStore, name: string): McpTool | undefined =>
    state.tools.find((tool) => tool.name === name),

  /**
   * Gets all tools provided by a specific server
   *
   * @param state - Tools store state
   * @param serverName - Server name to filter tools by
   * @returns Array of tools from the specified server
   */
  getToolsByServer: (state: ToolsStore, serverName: string): readonly McpTool[] =>
    state.tools.filter((tool) => tool.serverName === serverName),

  /**
   * Gets a server by its name
   *
   * @param state - Tools store state
   * @param name - Server name to look up
   * @returns McpServer if found, undefined otherwise
   */
  getServerByName: (state: ToolsStore, name: string): McpServer | undefined =>
    state.servers.find((server) => server.name === name),

  /**
   * Gets a server by its endpoint URL
   *
   * @param state - Tools store state
   * @param endpoint - Server endpoint to look up
   * @returns McpServer if found, undefined otherwise
   */
  getServerByEndpoint: (state: ToolsStore, endpoint: string): McpServer | undefined =>
    state.servers.find((server) => server.endpoint === endpoint),

  /**
   * Gets the total count of tools in the store
   *
   * @param state - Tools store state
   * @returns Number of tools
   */
  getToolCount: (state: ToolsStore): number => state.tools.length,

  /**
   * Gets the total count of servers in the store
   *
   * @param state - Tools store state
   * @returns Number of servers
   */
  getServerCount: (state: ToolsStore): number => state.servers.length,

  /**
   * Checks if any tools exist in the store
   *
   * @param state - Tools store state
   * @returns True if at least one tool exists, false otherwise
   */
  hasAnyTools: (state: ToolsStore): boolean => state.tools.length > 0,

  /**
   * Checks if any servers exist in the store
   *
   * @param state - Tools store state
   * @returns True if at least one server exists, false otherwise
   */
  hasAnyServers: (state: ToolsStore): boolean => state.servers.length > 0,

  /**
   * Checks if tools are currently being loaded
   *
   * @param state - Tools store state
   * @returns True if loading, false otherwise
   */
  isLoading: (state: ToolsStore): boolean => state.loading,

  /**
   * Checks if there is an error state
   *
   * @param state - Tools store state
   * @returns True if error exists, false otherwise
   */
  hasError: (state: ToolsStore): boolean => state.error !== null,

  /**
   * Gets the current error message
   *
   * @param state - Tools store state
   * @returns Error message or null
   */
  getError: (state: ToolsStore): string | null => state.error ?? null,
}
