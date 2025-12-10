import { create } from 'zustand'
import type { AgentCard } from '@a2a-js/sdk'

/**
 * Available MCP agents for the application.
 * Tracks loading state, errors, and user selection.
 */
interface AgentStore {
  agents: readonly AgentCard[]
  selectedAgentUrl: string | null
  selectedAgent: AgentCard | null
  loading: boolean
  error: string | null
  selectionError: string | null

  setAgents: (agents: AgentCard[]) => void
  addAgent: (agent: AgentCard) => void
  selectAgent: (agentUrl: string, agent: AgentCard) => void
  setLoading: (loading: boolean) => void
  setError: (error: string | null) => void
  setSelectionError: (error: string | null) => void
  clearSelection: () => void
}

/**
 * Store for managing available MCP agents and user selection.
 */
export const useAgentStore = create<AgentStore>()((set) => ({
  agents: [],
  selectedAgentUrl: null,
  selectedAgent: null,
  loading: false,
  error: null,
  selectionError: null,

  /**
   * Replace all agents with new list.
   * @param agents - Complete list of available agents
   */
  setAgents: (agents) => set({ agents }),

  /**
   * Add or update agent in the list, removing duplicate by URL.
   * @param agent - Agent to add or update
   */
  addAgent: (agent) =>
    set((state) => ({
      agents: [...state.agents.filter((a) => a.url !== agent.url), agent],
    })),

  /**
   * Select an agent for use.
   * @param agentUrl - URL of the agent
   * @param agent - Agent card details
   */
  selectAgent: (agentUrl, agent) => {
    set({
      selectedAgentUrl: agentUrl,
      selectedAgent: agent,
      selectionError: null,
    })
  },

  /**
   * Set loading state during agent fetch operations.
   * @param loading - Loading status
   */
  setLoading: (loading) => set({ loading }),

  /**
   * Set error state with optional error message.
   * @param error - Error message or null
   */
  setError: (error) => set({ error }),

  /**
   * Set agent selection error with optional error message.
   * @param error - Error message or null
   */
  setSelectionError: (error) => set({ selectionError: error }),

  /**
   * Clear agent selection and reset related state.
   */
  clearSelection: () =>
    set({
      selectedAgentUrl: null,
      selectedAgent: null,
      selectionError: null,
    }),
}))

/**
 * Selectors for reading agent state with type safety.
 */
export const agentSelectors = {
  /**
   * Get currently selected agent.
   * @param state - Current agent state
   * @returns Selected agent or null
   */
  getSelectedAgent: (state: AgentStore): AgentCard | null => state.selectedAgent ?? null,

  /**
   * Find agent by URL.
   * @param state - Current agent state
   * @param url - Agent URL
   * @returns Agent if found, undefined otherwise
   */
  getAgentByUrl: (state: AgentStore, url: string): AgentCard | undefined =>
    state.agents.find((a) => a.url === url),

  /**
   * Check if agents are currently being loaded.
   * @param state - Current agent state
   * @returns True if loading
   */
  isLoading: (state: AgentStore): boolean => state.loading,

  /**
   * Check if there's an error state.
   * @param state - Current agent state
   * @returns True if error exists
   */
  hasError: (state: AgentStore): boolean => state.error !== null,

  /**
   * Get error message if present.
   * @param state - Current agent state
   * @returns Error message or null
   */
  getError: (state: AgentStore): string | null => state.error ?? null,

  /**
   * Get total number of available agents.
   * @param state - Current agent state
   * @returns Agent count
   */
  getAgentCount: (state: AgentStore): number => state.agents.length,

  /**
   * Check if any agents are available.
   * @param state - Current agent state
   * @returns True if agents exist
   */
  hasAnyAgents: (state: AgentStore): boolean => state.agents.length > 0,

  /**
   * Check if an agent is currently selected.
   * @param state - Current agent state
   * @returns True if agent selected
   */
  isAgentSelected: (state: AgentStore): boolean => state.selectedAgent !== null,
}