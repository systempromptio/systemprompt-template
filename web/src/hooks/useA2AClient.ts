import { useState } from 'react'
import { useA2AClientInitialization } from './useA2AClientInitialization'
import { useA2AClientReady } from './useA2AClientReady'
import { useA2AMessageOperations } from './useA2AMessageOperations'

/**
 * Hook for managing A2A (Agent-to-Agent) client connections and operations.
 *
 * Composes three specialized hooks:
 * - useA2AClientInitialization: Client initialization with retry logic
 * - useA2AClientReady: Token validation and client readiness
 * - useA2AMessageOperations: Message and task operations
 *
 * @returns {Object} Agent client state and methods
 * @returns {A2AService | null} client - Connected client or null if not ready
 * @returns {boolean} loading - Whether initializing
 * @returns {Error | null} error - Last error encountered
 * @returns {boolean} retrying - Whether currently retrying
 * @returns {Function} retryConnection - Manually retry connection
 * @returns {Function} sendMessage - Send text message to agent
 * @returns {AsyncGenerator} streamMessage - Stream messages from agent
 * @returns {Function} getTask - Get specific task by ID
 * @returns {Function} cancelTask - Cancel specific task
 *
 * @example
 * ```typescript
 * const { client, loading, error, sendMessage } = useA2AClient()
 *
 * if (loading) return <div>Connecting...</div>
 * if (error) return <div>Error: {error.message}</div>
 *
 * const response = await sendMessage('Hello Agent')
 * ```
 */
export function useA2AClient() {
  const [error, setError] = useState<Error | null>(null)

  const {
    client,
    loading,
    error: initError,
    retrying,
    retryConnection
  } = useA2AClientInitialization()

  const { ensureClientReady } = useA2AClientReady(client, retryConnection)

  const {
    sendMessage,
    streamMessage,
    getTask,
    cancelTask
  } = useA2AMessageOperations(client, ensureClientReady, setError)

  return {
    client,
    loading,
    error: error || initError,
    retrying,
    retryConnection,
    sendMessage,
    streamMessage,
    getTask,
    cancelTask,
  }
}