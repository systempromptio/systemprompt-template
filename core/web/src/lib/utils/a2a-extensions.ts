import type { AgentCard } from '@a2a-js/sdk'
import { logger } from '@/lib/logger'

/**
 * SystemPrompt extension URIs
 */
export const SYSTEMPROMPT_EXTENSIONS = {
  AGENT_IDENTITY: 'systemprompt:agent-identity',
  SYSTEM_INSTRUCTIONS: 'systemprompt:system-instructions',
  SERVICE_STATUS: 'systemprompt:service-status',
  MCP_TOOLS: 'systemprompt:mcp-tools',
} as const

/**
 * Service status type
 */
export const ServiceStatus = {
  Running: 'Running',
  Stopped: 'Stopped',
  Starting: 'Starting',
  Stopping: 'Stopping',
  Failed: 'Failed',
  NotStarted: 'NotStarted',
  Unknown: 'Unknown',
} as const

export type ServiceStatus = typeof ServiceStatus[keyof typeof ServiceStatus]

/**
 * Get extension by URI
 */
export function getExtension(agent: AgentCard, uri: string) {
  return agent.capabilities?.extensions?.find((ext) => ext.uri === uri)
}

/**
 * Get extension parameter value
 */
export function getExtensionParam<T = any>(
  agent: AgentCard,
  uri: string,
  paramKey: string
): T | undefined {
  const extension = getExtension(agent, uri)
  if (!extension?.params) return undefined

  return extension.params[paramKey] as T
}

/**
 * Check if extension exists
 */
export function hasExtension(agent: AgentCard, uri: string): boolean {
  return !!getExtension(agent, uri)
}

/**
 * Get agent name from identity extension
 */
export function getAgentName(agent: AgentCard): string {
  return getExtensionParam<string>(
    agent,
    SYSTEMPROMPT_EXTENSIONS.AGENT_IDENTITY,
    'name'
  ) || agent.name
}

/**
 * Get system instructions from extension
 */
export function getSystemInstructions(agent: AgentCard): string | undefined {
  return getExtensionParam<string>(
    agent,
    SYSTEMPROMPT_EXTENSIONS.SYSTEM_INSTRUCTIONS,
    'systemPrompt'
  )
}

/**
 * Get service status from extension
 */
export interface ServiceStatusInfo {
  status: ServiceStatus
  port?: number
  pid?: number
  uptimeSeconds?: number
}

const VALID_STATUS_VALUES = ['running', 'stopped', 'starting', 'stopping', 'failed', 'notstarted'] as const

/**
 * Normalize status string - capitalize first letter to match ServiceStatus enum
 */
function normalizeStatus(status: string, agentName?: string): ServiceStatus {
  const normalized = status.charAt(0).toUpperCase() + status.slice(1).toLowerCase()

  switch (normalized) {
    case 'Running':
      return ServiceStatus.Running
    case 'Stopped':
      return ServiceStatus.Stopped
    case 'Starting':
      return ServiceStatus.Starting
    case 'Stopping':
      return ServiceStatus.Stopping
    case 'Failed':
      return ServiceStatus.Failed
    case 'Notstarted':
      return ServiceStatus.NotStarted
    default:
      logger.debug('Unexpected service status value', {
        agentName,
        receivedStatus: status,
        validStatuses: VALID_STATUS_VALUES,
      }, 'a2a-extensions')
      return ServiceStatus.Unknown
  }
}

export function getServiceStatus(agent: AgentCard): ServiceStatusInfo {
  const ext = getExtension(agent, SYSTEMPROMPT_EXTENSIONS.SERVICE_STATUS)

  if (!ext?.params) {
    return { status: ServiceStatus.Unknown }
  }

  const rawStatus = ext.params.status
  if (typeof rawStatus !== 'string') {
    logger.debug('Service status is not a string', {
      agentName: agent.name,
      receivedType: typeof rawStatus,
    }, 'a2a-extensions')
    return { status: ServiceStatus.Unknown }
  }

  const status = normalizeStatus(rawStatus, agent.name)

  return {
    status,
    port: typeof ext.params.port === 'number' ? ext.params.port : undefined,
    pid: typeof ext.params.pid === 'number' ? ext.params.pid : undefined,
    uptimeSeconds: typeof ext.params.uptimeSeconds === 'number' ? ext.params.uptimeSeconds : undefined,
  }
}

/**
 * Get MCP server information
 */
export interface McpServerInfo {
  name: string
  version: string
  transport: 'http' | 'stdio'
  enabled: boolean
}

interface RawMcpServer {
  name?: string
  version?: string
  endpoint?: string
  status?: string
}

function isValidMcpServer(value: unknown): value is RawMcpServer & { name: string; endpoint: string } {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Record<string, unknown>
  return typeof v.name === 'string' && typeof v.endpoint === 'string'
}

function isPartialMcpServer(value: unknown): value is RawMcpServer {
  return typeof value === 'object' && value !== null
}

export function getMcpServers(agent: AgentCard): McpServerInfo[] {
  const ext = getExtension(agent, SYSTEMPROMPT_EXTENSIONS.MCP_TOOLS)
  if (!ext?.params?.servers) return []

  const servers = ext.params.servers
  if (!Array.isArray(servers)) {
    logger.debug('MCP servers field is not an array', { agentName: agent.name }, 'a2a-extensions')
    return []
  }

  return servers
    .map((server, index): McpServerInfo | null => {
      if (isValidMcpServer(server)) {
        return {
          name: server.name,
          version: server.version || '1.0.0',
          transport: server.endpoint.startsWith('http') ? 'http' : 'stdio',
          enabled: server.status === 'running',
        }
      }

      // Log partial servers that are missing required fields
      if (isPartialMcpServer(server)) {
        logger.debug('MCP server config missing required fields', {
          agentName: agent.name,
          index,
          hasName: typeof server.name === 'string',
          hasEndpoint: typeof server.endpoint === 'string',
        }, 'a2a-extensions')
      }

      return null
    })
    .filter((server): server is McpServerInfo => server !== null)
}

/**
 * Check if agent requires OAuth (from spec security schemes)
 */
export function requiresOAuth(agent: AgentCard): boolean {
  return !!(
    agent.securitySchemes &&
    Object.keys(agent.securitySchemes).length > 0
  )
}

/**
 * Get required OAuth scopes (from spec security field)
 */
export function getRequiredScopes(agent: AgentCard): string[] {
  if (!agent.security || agent.security.length === 0) return []

  const allScopes = agent.security.flatMap((requirement) =>
    Object.values(requirement).flat()
  )

  return Array.from(new Set(allScopes))
}

/**
 * Permission hierarchy levels (matching backend Permission enum)
 */
function getPermissionLevel(scope: string): number {
  switch (scope.toLowerCase()) {
    case 'admin':
      return 100
    case 'user':
      return 50
    case 'service':
      return 40
    case 'a2a':
      return 30
    case 'mcp':
      return 20
    case 'anonymous':
      return 10
    default:
      return 0
  }
}

/**
 * Check if one permission implies another (hierarchical)
 * e.g., admin implies user, user implies anonymous
 */
function permissionImplies(userScope: string, requiredScope: string): boolean {
  return getPermissionLevel(userScope) >= getPermissionLevel(requiredScope)
}

/**
 * Check if user has any of the required scopes for an agent
 * Uses hierarchical permission checking: admin > user > anonymous
 * Agent accepts ANY of the listed scopes (OR logic)
 */
export function hasRequiredScopes(agent: AgentCard, userScopes: readonly string[]): boolean {
  const requiredScopes = getRequiredScopes(agent)
  if (requiredScopes.length === 0) return true

  // Agent accepts ANY of the required scopes (OR logic, not AND)
  return requiredScopes.some((requiredScope) =>
    userScopes.some((userScope) => permissionImplies(userScope, requiredScope))
  )
}

/**
 * Get all scopes that user doesn't have (for informational purposes)
 * Since agent accepts ANY scope, this shows which scopes the user is missing
 * Note: User only needs ONE of these to access, not all
 */
export function getMissingScopes(agent: AgentCard, userScopes: string[]): string[] {
  const requiredScopes = getRequiredScopes(agent)
  return requiredScopes.filter(
    (requiredScope) => !userScopes.some((userScope) => permissionImplies(userScope, requiredScope))
  )
}

/**
 * Format uptime duration
 */
export function formatUptime(seconds?: number): string {
  if (!seconds) return 'Unknown'

  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)

  if (hours > 24) {
    const days = Math.floor(hours / 24)
    return `${days}d ${hours % 24}h`
  }

  if (hours > 0) {
    return `${hours}h ${minutes}m`
  }

  return `${minutes}m`
}

/**
 * Get status badge color
 */
export function getStatusColor(status: ServiceStatus): string {
  switch (status) {
    case ServiceStatus.Running:
      return 'bg-success/20 text-success border-success/30'
    case ServiceStatus.Failed:
      return 'bg-error/20 text-error border-error/30'
    case ServiceStatus.Stopped:
      return 'bg-text-disabled/20 text-text-disabled border-text-disabled/30'
    case ServiceStatus.Starting:
    case ServiceStatus.Stopping:
      return 'bg-warning/20 text-warning border-warning/30'
    default:
      return 'bg-text-secondary/20 text-text-secondary border-text-secondary/30'
  }
}
