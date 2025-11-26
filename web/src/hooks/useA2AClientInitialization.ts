import { useEffect, useState, useCallback } from 'react'
import { A2AService, getA2AClient } from '@/lib/a2a/client'
import { useAgentStore } from '@/stores/agent.store'
import { useAuthStore } from '@/stores/auth.store'
import { logger } from '@/lib/logger'
import type { AgentCard } from '@a2a-js/sdk'

const MAX_RETRIES = 3

/**
 * Hook for managing A2A client initialization and connection retry logic.
 *
 * Handles:
 * - Client initialization with agent card or well-known endpoint
 * - Exponential backoff retry logic
 * - Agent card discovery and caching
 * - Connection state management
 *
 * @returns {Object} Initialization state and controls
 * @returns {A2AService | null} client - Connected A2A client or null
 * @returns {boolean} loading - Whether currently initializing
 * @returns {Error | null} error - Last error encountered
 * @returns {boolean} retrying - Whether currently in retry mode
 * @returns {Function} retryConnection - Manually trigger retry attempt
 *
 * @example
 * ```typescript
 * const { client, loading, error } = useA2AClientInitialization()
 * if (loading) return <LoadingSpinner />
 * if (error) return <ErrorBanner message={error.message} />
 * ```
 */
export function useA2AClientInitialization() {
  const [client, setClient] = useState<A2AService | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<Error | null>(null)
  const [retrying, setRetrying] = useState(false)

  const selectedAgentUrl = useAgentStore((state) => state.selectedAgentUrl)
  const selectedAgent = useAgentStore((state) => state.selectedAgent)
  const agents = useAgentStore((state) => state.agents)
  const selectAgent = useAgentStore((state) => state.selectAgent)
  const accessToken = useAuthStore((state) => state.accessToken)

  const initializeClientInternal = useCallback(async (
    agentUrl: string,
    agentCard: AgentCard | null,
    authHeader: string | null
  ): Promise<A2AService> => {
    const service = getA2AClient(agentUrl, authHeader)

    if (agentCard) {
      try {
        await service.initialize(agentCard)
        logger.debug('Successfully initialized with existing card', undefined, 'useA2AClientInitialization')
        return service
      } catch (err) {
        logger.error('Failed to initialize with existing card', err, 'useA2AClientInitialization')
        throw err
      }
    }

    if (agentUrl.includes('/api/v1/agents/')) {
      const err = new Error('Agent card not found for proxy URL. Please refresh agent discovery.')
      logger.error('Agent card not found for proxy URL', err, 'useA2AClientInitialization')
      throw err
    }

    try {
      const fetchedCard = await service.initialize()
      const card: AgentCard = {
        ...fetchedCard,
        url: agentUrl
      }
      selectAgent(agentUrl, card)
      logger.debug('Successfully initialized from well-known', undefined, 'useA2AClientInitialization')
      return service
    } catch (err) {
      logger.error('Failed to initialize from well-known', err, 'useA2AClientInitialization')
      throw err
    }
  }, [selectAgent])

  useEffect(() => {
    if (!selectedAgentUrl) {
      setClient(null)
      return
    }

    const authHeader = accessToken ? `Bearer ${accessToken}` : null
    setLoading(true)
    setError(null)

    let agentCard = selectedAgent

    if (!agentCard && agents.length > 0) {
      agentCard = agents.find(a => a.url === selectedAgentUrl) || null
      if (agentCard) {
        selectAgent(selectedAgentUrl, agentCard)
      }
    }

    const attemptInitialization = async (attempt: number) => {
      try {
        const initializedClient = await initializeClientInternal(selectedAgentUrl, agentCard, authHeader)
        setClient(initializedClient)
        setLoading(false)
        setRetrying(false)
        setError(null)
      } catch (err) {
        if (attempt < MAX_RETRIES) {
          const nextAttempt = attempt + 1
          const retryDelay = Math.min(1000 * Math.pow(2, nextAttempt), 5000)
          logger.debug('Retrying agent connection', { attempt: nextAttempt, maxRetries: MAX_RETRIES }, 'useA2AClientInitialization')

          setRetrying(true)
          setError(new Error(`Connection failed. Retrying (${nextAttempt}/${MAX_RETRIES})...`))

          await new Promise(resolve => setTimeout(resolve, retryDelay))
          await attemptInitialization(nextAttempt)
        } else {
          logger.error('Max retries reached, initialization failed', err, 'useA2AClientInitialization')
          setError(new Error('Failed to connect to agent. Please refresh the page or try again later.'))
          setClient(null)
          setLoading(false)
          setRetrying(false)
        }
      }
    }

    attemptInitialization(0)
  }, [selectedAgentUrl, selectedAgent, agents, selectAgent, accessToken, initializeClientInternal])

  const retryConnection = useCallback(() => {
    if (!selectedAgentUrl) return

    setError(null)
    setClient(null)
    setLoading(true)

    const authHeader = accessToken ? `Bearer ${accessToken}` : null
    let agentCard = selectedAgent

    if (!agentCard && agents.length > 0) {
      agentCard = agents.find(a => a.url === selectedAgentUrl) || null
    }

    const attemptInitialization = async () => {
      try {
        const initializedClient = await initializeClientInternal(selectedAgentUrl, agentCard, authHeader)
        setClient(initializedClient)
        setLoading(false)
        setRetrying(false)
        setError(null)
        logger.debug('Retry successful, client reinitialized', undefined, 'useA2AClientInitialization')
      } catch (err) {
        const errorToSet = err instanceof Error ? err : new Error(typeof err === 'string' ? err : 'Connection failed')
        setError(errorToSet)
        setClient(null)
        setLoading(false)
        setRetrying(false)
      }
    }

    attemptInitialization()
  }, [selectedAgentUrl, selectedAgent, agents, accessToken, initializeClientInternal])

  return {
    client,
    loading,
    error,
    retrying,
    retryConnection
  }
}
