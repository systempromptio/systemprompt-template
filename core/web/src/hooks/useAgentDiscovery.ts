import { useEffect, useCallback, useRef } from 'react'
import { A2AClient } from '@a2a-js/sdk/client'
import type { AgentCard } from '@a2a-js/sdk'
import { useAgentStore } from '@/stores/agent.store'
import { useContextStore } from '@/stores/context.store'
import { useAuthStore } from '@/stores/auth.store'
import { useSkillStore } from '@/stores/skill.store'
import { logger } from '@/lib/logger'

export interface AgentEndpoint {
  url: string
  name: string
  description?: string
}

/**
 * Fetches the agent registry from the API.
 *
 * Connects to /api/v1/agents/registry and returns the agent cards list.
 * Returns empty array on error rather than throwing.
 *
 * @returns Promise resolving to array of AgentCard objects
 */
const fetchAgentsFromAPI = async (): Promise<AgentCard[]> => {
  try {
    const authHeader = useAuthStore.getState().getAuthHeader()
    if (!authHeader) {
      logger.error('Missing authentication', new Error('No JWT token available'), 'useAgentDiscovery')
      throw new Error('Missing authentication')
    }

    logger.debug('Fetching agents from registry API', undefined, 'useAgentDiscovery')
    const response = await fetch('/api/v1/agents/registry', {
      headers: {
        'Authorization': authHeader,
      },
    })
    if (!response.ok) {
      logger.error('Failed to fetch agents', new Error(`API returned ${response.status}`), 'useAgentDiscovery')
      throw new Error(`API returned ${response.status}`)
    }
    const data = await response.json()

    if (data.data && Array.isArray(data.data)) {
      const agents = data.data as AgentCard[]
      logger.debug('Successfully loaded agents', { count: agents.length }, 'useAgentDiscovery')
      return agents
    }

    logger.warn('No agents found in response', undefined, 'useAgentDiscovery')
    return []
  } catch (error) {
    logger.error('Failed to fetch agents from registry API', error, 'useAgentDiscovery')
    return []
  }
}

/**
 * Discovers and manages available agents.
 *
 * Loads agents from the registry on mount if none are cached.
 * Supports custom agent discovery via endpoint URL.
 *
 * @returns Object with refresh and addCustomAgent methods
 *
 * @throws {Error} When registry API request fails
 * @throws {Error} When agent card fetch from custom endpoint fails
 * @throws {Error} When agent endpoint is not reachable
 * @throws {Error} When agent response is invalid JSON
 *
 * @example
 * ```typescript
 * const { refresh, addCustomAgent } = useAgentDiscovery()
 * await refresh()
 * await addCustomAgent('https://agent.example.com')
 * ```
 */
export function useAgentDiscovery() {
  const {
    agents,
    selectedAgent,
    setAgents,
    addAgent,
    selectAgent,
    setLoading,
    setError
  } = useAgentStore()
  const hasAttemptedLoad = useRef(false)

  /**
   * Discovers a single agent by connecting to its endpoint.
   *
   * Attempts well-known card endpoint first, falls back to alternate
   * URL if that fails. Proxy paths fail fast.
   *
   * @param endpoint - Agent endpoint with URL and name
   * @returns Promise resolving to AgentCard or null if discovery fails
   */
  const discoverAgent = useCallback(
    async (endpoint: AgentEndpoint): Promise<AgentCard | null> => {
      try {
        let client: A2AClient
        const isProxyPath = endpoint.url.includes('/server/')

        try {
          const cardUrl = `${endpoint.url}/.well-known/agent-card.json`
          client = await A2AClient.fromCardUrl(cardUrl)
        } catch {
          if (!isProxyPath) {
            const fallbackUrl = `${endpoint.url}/api/agents/card`
            client = await A2AClient.fromCardUrl(fallbackUrl)
          } else {
            throw new Error('Failed to fetch agent card through proxy')
          }
        }

        const card = await client.getAgentCard()
        const agentCard: AgentCard = {
          ...card,
          url: endpoint.url
        }
        logger.debug('Successfully discovered agent', { name: card.name }, 'useAgentDiscovery')
        return agentCard
      } catch (error) {
        logger.error(`Failed to discover agent at endpoint`, error, 'useAgentDiscovery')
        return null
      }
    },
    []
  )

  /**
   * Discovers all agents from the registry and auto-selects a default.
   *
   * Fetches all agents from registry API and stores them. If no agent is
   * currently selected, auto-selects first agent marked as default, or first
   * agent in list as fallback.
   *
   * @returns Promise that resolves when discovery completes (or fails)
   */
  const discoverAllAgents = useCallback(async () => {
    hasAttemptedLoad.current = true
    setLoading(true)
    setError(null)

    try {
      const apiAgents = await fetchAgentsFromAPI()

      if (apiAgents.length > 0) {
        logger.debug('Setting agents in store', { count: apiAgents.length }, 'useAgentDiscovery')
        setAgents(apiAgents)

        // Load skills from agent cards (skills were loaded from DB into agent cards by backend)
        const allSkills = apiAgents.flatMap(agent => {
          if ('skills' in agent && Array.isArray(agent.skills)) {
            return agent.skills
          }
          return []
        })
        if (allSkills.length > 0) {
          logger.debug('Loading skills from agent cards', { count: allSkills.length }, 'useAgentDiscovery')
          useSkillStore.getState().loadSkills(allSkills)
        }

        // Retry pending agent selection after agents are loaded
        const contextStore = useContextStore.getState()
        const currentContextId = contextStore.currentContextId
        const assignedAgentName = currentContextId !== 'LOADING' && currentContextId !== 'NONE'
          ? contextStore.contextAgents.get(currentContextId)
          : null

        if (assignedAgentName) {
          // There's a pending agent assignment from SSE - use it
          const matchingAgent = apiAgents.find((agent: AgentCard) =>
            agent.name.toLowerCase() === assignedAgentName.toLowerCase()
          )
          if (matchingAgent) {
            selectAgent(matchingAgent.url, matchingAgent)
          } else {
            logger.warn('Assigned agent not found', { agentName: assignedAgentName }, 'useAgentDiscovery')
          }
        } else if (!selectedAgent && apiAgents.length > 0) {
          // No assignment - use default agent
          const defaultAgent = apiAgents.find((agent: AgentCard) => {
            const serviceStatusExt = agent.capabilities?.extensions?.find(
              (ext: { uri: string }) => ext.uri === 'systemprompt:service-status'
            )
            return serviceStatusExt?.params?.default === true
          })
          const agentToSelect = defaultAgent || apiAgents[0]
          selectAgent(agentToSelect.url, agentToSelect)
        }
      } else {
        logger.warn('No agents found in registry', undefined, 'useAgentDiscovery')
        setError('No agents found in registry')
      }
    } catch (err) {
      logger.error('Agent discovery error', err, 'useAgentDiscovery')
      setError('Failed to discover agents from registry')
    } finally {
      setLoading(false)
    }
  }, [setAgents, selectAgent, setLoading, setError])

  /**
   * Discovers and adds a custom agent by URL.
   *
   * Attempts to discover an agent at the given URL and adds it to the
   * available agents list. Sets error state if discovery fails.
   *
   * @param url - Agent endpoint URL to discover
   * @returns Promise resolving to true if successful, false otherwise
   */
  const addCustomAgent = useCallback(
    async (url: string): Promise<boolean> => {
      setLoading(true)
      setError(null)

      try {
        const agent = await discoverAgent({
          url,
          name: 'Custom Agent',
          description: 'User-added agent',
        })

        if (agent) {
          logger.debug('Custom agent added successfully', { name: agent.name }, 'useAgentDiscovery')
          addAgent(agent)
          return true
        } else {
          logger.error('Failed to discover custom agent', new Error(`Failed at URL: ${url}`), 'useAgentDiscovery')
          setError(`Failed to connect to agent at ${url}`)
          return false
        }
      } catch (err) {
        logger.error('Error connecting to agent', err, 'useAgentDiscovery')
        setError(`Error connecting to agent: ${err}`)
        return false
      } finally {
        setLoading(false)
      }
    },
    [discoverAgent, addAgent]
  )

  useEffect(() => {
    if (agents.length === 0 && !hasAttemptedLoad.current) {
      discoverAllAgents()
    }
  }, [agents.length, discoverAllAgents])

  return {
    refresh: discoverAllAgents,
    addCustomAgent,
  }
}