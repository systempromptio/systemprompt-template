/**
 * Hook for initializing A2A client connection.
 *
 * Handles client setup, configuration, and initial connection
 * to the A2A server. Manages connection lifecycle.
 *
 * @module hooks/a2a/useA2AInitializer
 */

import { useState, useCallback, useEffect } from 'react'
import { A2AService, getA2AClient } from '@/lib/a2a/client'
import { useAgentStore } from '@/stores/agent.store'
import { useAuthStore } from '@/stores/auth.store'
import { useContextStore } from '@/stores/context.store'
import { logger } from '@/lib/logger'
import type { AgentCard } from '@a2a-js/sdk'

const MAX_RETRIES = 3

/**
 * A2A client initialization state.
 */
interface InitializationState {
  /**
   * Initialized A2A client instance
   */
  client: A2AService | null

  /**
   * Whether client is currently initializing
   */
  isInitializing: boolean

  /**
   * Whether client is connected and ready
   */
  isReady: boolean

  /**
   * Initialization error, if any
   */
  error: Error | null

  /**
   * Current retry attempt number
   */
  retryCount: number

  /**
   * Whether currently retrying
   */
  isRetrying: boolean
}

/**
 * Initializer hook return value.
 */
interface UseA2AInitializerReturn extends InitializationState {
  /**
   * Manually retry the client initialization
   */
  retryConnection: () => void

  /**
   * Disconnect and cleanup the client
   */
  disconnect: () => void
}

/**
 * Initializes and manages A2A client lifecycle.
 *
 * Handles client creation, connection, and cleanup. Automatically
 * initializes when agent is selected and retries with exponential
 * backoff on failure.
 *
 * @returns Client initialization state and controls
 *
 * @example
 * ```typescript
 * function AgentInterface() {
 *   const { client, isReady, retryConnection } = useA2AInitializer()
 *
 *   if (!isReady) return <Loading />
 *   return <Chat client={client} />
 * }
 * ```
 *
 * @throws {Error} When initialization fails after max retries
 */
export function useA2AInitializer(): UseA2AInitializerReturn {
  const selectedAgentUrl = useAgentStore((state) => state.selectedAgentUrl)
  const selectedAgent = useAgentStore((state) => state.selectedAgent)
  const agents = useAgentStore((state) => state.agents)
  const selectAgent = useAgentStore((state) => state.selectAgent)
  const accessToken = useAuthStore((state) => state.accessToken)
  const currentContextId = useContextStore((state) => state.currentContextId)

  const [state, setState] = useState<InitializationState>({
    client: null,
    isInitializing: false,
    isReady: false,
    error: null,
    retryCount: 0,
    isRetrying: false,
  })

  /**
   * Initializes the A2A service with provided agent card or by fetching from well-known endpoint.
   */
  const initializeClient = useCallback(
    async (agentUrl: string, agentCard: AgentCard | null, authHeader: string | null): Promise<A2AService> => {
      const service = getA2AClient(agentUrl, authHeader)

      if (agentCard) {
        await service.initialize(agentCard)
        logger.debug('Successfully initialized with existing card', undefined, 'useA2AInitializer')
        return service
      } else if (agentUrl.includes('/api/v1/agents/')) {
        throw new Error('Agent card not found for proxy URL. Please refresh agent discovery.')
      } else {
        const fetchedCard = await service.initialize()
        const card: AgentCard = {
          ...fetchedCard,
          url: agentUrl,
        }
        selectAgent(agentUrl, card)
        logger.debug('Successfully initialized from well-known', undefined, 'useA2AInitializer')
        return service
      }
    },
    [selectAgent]
  )

  /**
   * Attempts initialization with exponential backoff retry logic.
   */
  const attemptInitialization = useCallback(
    async (isManualRetry: boolean = false) => {
      if (!selectedAgentUrl) {
        setState((prev) => ({
          ...prev,
          client: null,
          retryCount: 0,
        }))
        return
      }

      const authHeader = accessToken ? `Bearer ${accessToken}` : null
      setState((prev) => ({
        ...prev,
        isInitializing: !isManualRetry,
        error: null,
      }))

      let agentCard = selectedAgent

      if (!agentCard && agents.length > 0) {
        agentCard = agents.find((a) => a.url === selectedAgentUrl) || null
        if (agentCard) {
          selectAgent(selectedAgentUrl, agentCard)
        }
      }

      try {
        const client = await initializeClient(selectedAgentUrl, agentCard, authHeader)
        setState((prev) => ({
          ...prev,
          client,
          isInitializing: false,
          isReady: true,
          isRetrying: false,
          error: null,
          retryCount: 0,
        }))
      } catch (err) {
        setState((prev) => {
          const nextRetryCount = prev.retryCount + 1

          if (nextRetryCount < MAX_RETRIES) {
            const retryDelay = Math.min(1000 * Math.pow(2, nextRetryCount), 5000)
            logger.debug('Retrying agent connection', { attempt: nextRetryCount, maxRetries: MAX_RETRIES }, 'useA2AInitializer')

            setTimeout(() => {
              attemptInitialization(false)
            }, retryDelay)

            return {
              ...prev,
              isRetrying: true,
              isInitializing: false,
              error: new Error(`Connection failed. Retrying (${nextRetryCount}/${MAX_RETRIES})...`),
              retryCount: nextRetryCount,
            }
          } else {
            logger.error('Max retries reached, initialization failed', err, 'useA2AInitializer')
            return {
              ...prev,
              client: null,
              isInitializing: false,
              isReady: false,
              isRetrying: false,
              error: new Error('Failed to connect to agent. Please refresh the page or try again later.'),
              retryCount: nextRetryCount,
            }
          }
        })
      }
    },
    [selectedAgentUrl, selectedAgent, agents, selectAgent, accessToken, initializeClient]
  )

  /**
   * Setup automatic initialization when agent or context changes.
   */
  useEffect(() => {
    attemptInitialization(false)
  }, [selectedAgentUrl, currentContextId, attemptInitialization])

  const retryConnection = useCallback(() => {
    setState((prev) => ({
      ...prev,
      retryCount: 0,
      error: null,
      client: null,
      isInitializing: true,
    }))
    attemptInitialization(true)
  }, [attemptInitialization])

  const disconnect = useCallback(() => {
    if (state.client) {
      ;(state.client as any).disconnect?.()
    }
    setState((prev) => ({
      ...prev,
      client: null,
      isInitializing: false,
      isReady: false,
      isRetrying: false,
      error: null,
      retryCount: 0,
    }))
    logger.debug('A2A client disconnected', undefined, 'useA2AInitializer')
  }, [state.client])

  useEffect(() => {
    return () => {
      disconnect()
    }
  }, [disconnect])

  return { ...state, retryConnection, disconnect }
}
