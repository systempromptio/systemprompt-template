/**
 * Hooks Barrel Export
 *
 * Central export for all application hooks organized by category.
 * Simplifies imports: `import { useA2AClient, useMcpRegistry } from '@/hooks'`
 */

// Authentication & Token Management
export { useAuth } from './useAuth'
export { useTokenExpiryMonitor } from './useTokenExpiryMonitor'

// Agent & MCP Management
export { useAgentDiscovery } from './useAgentDiscovery'
export { useA2AClient } from './useA2AClient'
export { useMcpRegistry } from './useMcpRegistry'
export { useMcpToolCaller } from './useMcpToolCaller'
export { useToolParameters } from './useToolParameters'

// Context & Streaming
export { useContextInit } from './useContextInit'
export { useContextStream } from './useContextStream'
export { useSSEConnection } from './useSSEConnection'
export { useStreamEventProcessor } from './useStreamEventProcessor'
export { useArtifactSubscription } from './useArtifactSubscription'

// Data & Options
export { useDynamicOptions } from './useDynamicOptions'
export { useResolvedSchema } from './useResolvedSchema'

// Utilities & State Management
export { useRetry } from './useRetry'
export { useAsyncState } from './useAsyncState'

// Type exports for convenience
export type { UseRetryOptions, UseRetryResult } from './useRetry'
export type { AsyncState, UseAsyncStateOptions } from './useAsyncState'
export type { UseSSEConnectionOptions, UseSSEConnectionResult } from './useSSEConnection'
export type { AgentEndpoint } from './useAgentDiscovery'
export type { DynamicOption, UseDynamicOptionsResult } from './useDynamicOptions'
