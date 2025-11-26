import { create } from 'zustand'
import { contextsService } from '@/services/contexts.service'
import { useChatStore } from './chat.store'
import { useAuthStore } from './auth.store'
import { useAgentStore } from './agent.store'
import { logger } from '@/lib/logger'
import {
  getStorageKey,
  loadFromStorage,
  saveToStorage,
  setError,
  clearError,
} from './store-utilities'

const getContextStorageKey = (userId?: string): string => {
  return getStorageKey('context', userId)
}

const loadPersistedContextId = (): string | undefined => {
  const userId = useAuthStore.getState().userId
  const key = getContextStorageKey(userId || undefined)
  return loadFromStorage(key)
}

const persistContextId = (id: string): void => {
  const userId = useAuthStore.getState().userId
  const key = getContextStorageKey(userId || undefined)
  saveToStorage(key, id)
}

const getAuthToken = (): string => {
  const { accessToken, isTokenValid } = useAuthStore.getState()

  if (!accessToken || !isTokenValid()) {
    throw new Error('Authentication required: No valid JWT token available')
  }

  return `Bearer ${accessToken}`
}

export interface Conversation {
  id: string
  name: string
  createdAt: Date
  updatedAt: Date
  messageCount: number
}

interface ContextStateEvent {
  type: string
  context_id?: string
  timestamp: string
  [key: string]: any
}

interface ContextStore {
  conversations: Map<string, Conversation>
  currentContextId: string
  isLoading: boolean
  error: string | null
  hasReceivedSnapshot: boolean
  contextAgents: Map<string, string>

  sseStatus: 'connected' | 'connecting' | 'disconnected' | 'error'
  sseError: string | null

  conversationList: () => Conversation[]
  getCurrentConversation: () => Conversation | null
  createConversation: (name?: string) => Promise<void>
  switchConversation: (id: string) => void
  renameConversation: (id: string, name: string) => Promise<void>
  deleteConversation: (id: string) => Promise<void>
  updateMessageCount: (id: string) => void
  clearError: () => void

  setSSEStatus: (status: 'connected' | 'connecting' | 'disconnected' | 'error') => void
  setSSEError: (error: string | null) => void
  handleSnapshot: (contexts: Array<{context_id: string, name: string, created_at: string, updated_at: string, message_count: number}>) => void
  handleStateEvent: (event: ContextStateEvent) => void
}


export const CONTEXT_STATE = {
  LOADING: '__CONTEXT_LOADING__',
} as const

export const useContextStore = create<ContextStore>()((set, get) => ({
  conversations: new Map(),
  currentContextId: CONTEXT_STATE.LOADING,
  isLoading: true,
  error: null,
  hasReceivedSnapshot: false,
  contextAgents: new Map(),

  sseStatus: 'disconnected',
  sseError: null,

  conversationList: () => {
    const conversations = Array.from(get().conversations.values())
    return conversations.sort((a, b) => b.updatedAt.getTime() - a.updatedAt.getTime())
  },

  getCurrentConversation: () => {
    const { currentContextId, conversations } = get()

    if (currentContextId === CONTEXT_STATE.LOADING) {
      return null
    }

    const conv = conversations.get(currentContextId)
    if (!conv) {
      return null
    }
    return conv
  },

  createConversation: async (name?: string) => {
    const conversationName = name || (() => {
      const existingConversations = Array.from(get().conversations.values())
      const conversationNumbers = existingConversations
        .map(c => {
          const match = c.name.match(/^Conversation (\d+)$/)
          return match ? parseInt(match[1], 10) : 0
        })
        .filter(n => n > 0)

      const maxNumber = conversationNumbers.length > 0
        ? Math.max(...conversationNumbers)
        : 0

      return `Conversation ${maxNumber + 1}`
    })()

    try {
      const authToken = getAuthToken()
      const { context, error } = await contextsService.createContext(conversationName, authToken)

      if (error) {
        logger.error('Failed to create conversation', error, 'ContextStore')
        set(setError(`Failed to create conversation: ${error}`))
        return
      }

      if (context) {
        const newConversation: Conversation = {
          id: context.context_id,
          name: context.name,
          createdAt: new Date(context.created_at),
          updatedAt: new Date(context.updated_at),
          messageCount: 0,
        }

        set((state) => {
          const updated = new Map(state.conversations)
          updated.set(context.context_id, newConversation)
          return {
            conversations: updated,
            currentContextId: context.context_id,
            ...clearError(),
          }
        })

        persistContextId(context.context_id)
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Unknown error'
      logger.error('Error creating conversation', err, 'ContextStore')
      set(setError(`Failed to create conversation: ${message}`))
    }
  },

  switchConversation: (id: string) => {
    const state = get()
    const conversations = state.conversations
    if (!conversations.has(id)) {
      logger.error('Cannot switch to non-existent context', { id }, 'ContextStore')
      set(setError(`Cannot switch to non-existent context: ${id}`))
      return
    }

    const agentName = state.contextAgents.get(id)
    if (agentName) {
      const agentStore = useAgentStore.getState()
      const matchingAgent = agentStore.agents.find(
        agent => agent.name === agentName
      )

      if (matchingAgent) {
        agentStore.selectAgent(matchingAgent.url, matchingAgent)
        logger.debug('Auto-selected agent for switched context', {
          contextId: id,
          agentName
        }, 'ContextStore')
      }
    }

    persistContextId(id)
    set({ currentContextId: id, ...clearError() })
  },

  renameConversation: async (id: string, name: string) => {
    const conversations = get().conversations
    if (!conversations.has(id)) {
      logger.error('Cannot rename non-existent context', { id }, 'ContextStore')
      set(setError(`Cannot rename non-existent context: ${id}`))
      return
    }

    const previousState = conversations.get(id)!

    set((state) => {
      const updated = new Map(state.conversations)
      const conv = updated.get(id)!
      updated.set(id, { ...conv, name, updatedAt: new Date() })
      return { conversations: updated, ...clearError() }
    })

    try {
      const authToken = getAuthToken()
      await contextsService.updateContext(id, name, authToken)
    } catch (error) {
      logger.error('Failed to rename conversation', error, 'ContextStore')
      set((state) => {
        const updated = new Map(state.conversations)
        updated.set(id, previousState)
        return {
          conversations: updated,
          ...setError('Failed to rename conversation')
        }
      })
    }
  },

  deleteConversation: async (id: string) => {
    const state = get()

    if (!state.conversations.has(id)) {
      logger.error('Cannot delete non-existent context', { id }, 'ContextStore')
      set(setError(`Cannot delete non-existent context: ${id}`))
      return
    }

    const updated = new Map(state.conversations)
    updated.delete(id)

    const remainingIds = Array.from(updated.keys())
    const newCurrentId = state.currentContextId === id
      ? remainingIds[0]
      : state.currentContextId

    if (!newCurrentId) {
      logger.error('Cannot delete last context', undefined, 'ContextStore')
      set(setError('Cannot delete last context - system must always have at least one context'))
      return
    }

    const previousState = {
      conversations: state.conversations,
      currentContextId: state.currentContextId,
    }

    persistContextId(newCurrentId)

    set({
      conversations: updated,
      currentContextId: newCurrentId,
      ...clearError(),
    })

    try {
      const authToken = getAuthToken()
      await contextsService.deleteContext(id, authToken)
    } catch (error) {
      logger.error('Failed to delete conversation', error, 'ContextStore')
      persistContextId(previousState.currentContextId)
      set({
        conversations: previousState.conversations,
        currentContextId: previousState.currentContextId,
        ...setError('Failed to delete conversation')
      })
    }
  },

  updateMessageCount: (id: string) => {
    const conversations = get().conversations
    if (!conversations.has(id)) {
      logger.warn('updateMessageCount called for non-existent context', { id }, 'ContextStore')
      return
    }

    set((state) => {
      const updated = new Map(state.conversations)
      const conv = updated.get(id)!
      updated.set(id, { ...conv, messageCount: conv.messageCount + 1, updatedAt: new Date() })
      return { conversations: updated }
    })
  },

  clearError: () => set({ error: null }),

  setSSEStatus: (status) => set({ sseStatus: status }),
  setSSEError: (error) => set({ sseError: error }),

  handleSnapshot: (contexts: Array<{context_id: string, name: string, created_at: string, updated_at: string, message_count: number}>) => {
    logger.debug('Received snapshot', { count: contexts.length }, 'ContextStore')

    if (contexts.length === 0) {
      logger.debug('Empty snapshot - useContextInit will create default context', undefined, 'ContextStore')
      set({
        conversations: new Map(),
        currentContextId: CONTEXT_STATE.LOADING,
        isLoading: false,
        hasReceivedSnapshot: true,
      })
      return
    }

    const conversationsArray = contexts.map((ctx): Conversation => ({
      id: ctx.context_id,
      name: ctx.name,
      createdAt: new Date(ctx.created_at),
      updatedAt: new Date(ctx.updated_at),
      messageCount: ctx.message_count,
    }))

    const conversations = new Map(conversationsArray.map(c => [c.id, c]))
    const persistedId = loadPersistedContextId()

    const validPersistedId = persistedId && conversations.has(persistedId)
      ? persistedId
      : undefined

    const sortedArray = conversationsArray.sort((a, b) => b.updatedAt.getTime() - a.updatedAt.getTime())
    const selectedId = validPersistedId || sortedArray[0].id

    persistContextId(selectedId)

    set({
      conversations,
      currentContextId: selectedId,
      isLoading: false,
      hasReceivedSnapshot: true,
    })

    // Auto-select agent if we have one stored for this context
    const agentName = get().contextAgents.get(selectedId)
    if (agentName) {
      const agentStore = useAgentStore.getState()
      const matchingAgent = agentStore.agents.find(
        agent => agent.name === agentName
      )

      if (matchingAgent) {
        agentStore.selectAgent(matchingAgent.url, matchingAgent)
        logger.debug('Auto-selected agent for initial context', {
          contextId: selectedId,
          agentName
        }, 'ContextStore')
      }
    }
  },

  handleStateEvent: (event) => {
    console.log('[contextStore.handleStateEvent] Received:', { type: event.type, contextId: event.context_id, fullEvent: event })
    logger.debug('State event received', { type: event.type, contextId: event.context_id }, 'ContextStore')

    switch (event.type) {
      case 'message_added':
        break

      case 'tool_execution_completed':
        break

      case 'task_completed':
        break

      case 'task_created':
        break

      case 'task_status_changed':
        if ('task_id' in event && event.context_id) {
          useChatStore.getState().handleTaskStatusChanged?.({
            context_id: event.context_id,
            task_id: event.task_id,
            status: event.status,
            timestamp: event.timestamp
          })
        }
        break

      case 'context_created':
        if ('context' in event && event.context_id) {
          const newConversation: Conversation = {
            id: event.context.context_id,
            name: event.context.name,
            createdAt: new Date(event.context.created_at),
            updatedAt: new Date(event.context.updated_at),
            messageCount: 0,
          }
          set((state) => {
            const updated = new Map(state.conversations)
            updated.set(event.context.context_id, newConversation)
            return {
              conversations: updated,
              currentContextId: state.currentContextId === CONTEXT_STATE.LOADING
                ? event.context_id
                : state.currentContextId,
            }
          })
        }
        break

      case 'context_updated':
        if ('name' in event && event.context_id) {
          set((state) => {
            const updated = new Map(state.conversations)
            const conv = updated.get(event.context_id!)
            if (conv) {
              updated.set(event.context_id!, { ...conv, name: event.name, updatedAt: new Date(event.timestamp) })
            }
            return { conversations: updated }
          })
        }
        break

      case 'context_deleted':
        if (event.context_id) {
          set((state) => {
            const updated = new Map(state.conversations)
            updated.delete(event.context_id!)

            const newCurrentId = state.currentContextId === event.context_id
              ? Array.from(updated.keys())[0] || CONTEXT_STATE.LOADING
              : state.currentContextId

            return {
              conversations: updated,
              currentContextId: newCurrentId
            }
          })
        }
        break

      case 'heartbeat':
        break

      case 'artifact_created':
        break

      case 'current_agent':
        console.log('[contextStore] Processing current_agent event:', event)
        if ('context_id' in event && event.context_id) {
          const contextId = event.context_id
          const agentName = 'agent_name' in event ? event.agent_name : null

          set((state) => {
            const updated = new Map(state.contextAgents)
            if (agentName) {
              updated.set(contextId, agentName)
            } else {
              updated.delete(contextId)
            }
            return { contextAgents: updated }
          })

          if (agentName) {
            const agentStore = useAgentStore.getState()
            console.log('[contextStore] Looking for agent:', agentName)
            console.log('[contextStore] Available agents:', agentStore.agents.map(a => ({ name: a.name, url: a.url })))

            const matchingAgent = agentStore.agents.find(
              agent => agent.name.toLowerCase() === agentName.toLowerCase()
            )

            console.log('[contextStore] Matching agent found:', matchingAgent)

            if (matchingAgent) {
              const currentContextId = get().currentContextId
              console.log('[contextStore] Current context ID:', currentContextId, 'Event context ID:', contextId)
              if (currentContextId === contextId) {
                console.log('[contextStore] Calling selectAgent with:', matchingAgent.url, matchingAgent.name)
                agentStore.selectAgent(matchingAgent.url, matchingAgent)
                logger.debug('Auto-selected agent for current context', {
                  contextId,
                  agentName
                }, 'ContextStore')
              }
            } else {
              console.warn('[contextStore] No matching agent found for:', agentName, '(agents may not be loaded yet)')
            }
          }
        }
        break

      default:
        logger.warn('Unknown event type', { type: event.type }, 'ContextStore')
    }
  },
}))

export const contextSelectors = {
  getCurrentContextId: (state: ContextStore): string =>
    state.currentContextId,

  isContextLoading: (state: ContextStore): boolean =>
    state.currentContextId === CONTEXT_STATE.LOADING,

  getCurrentConversation: (state: ContextStore): Conversation | null =>
    state.getCurrentConversation() ?? null,

  getConversationById: (state: ContextStore, id: string): Conversation | undefined =>
    state.conversations.get(id),

  getConversationList: (state: ContextStore): readonly Conversation[] =>
    state.conversationList(),

  getConversationCount: (state: ContextStore): number =>
    state.conversations.size,

  isLoading: (state: ContextStore): boolean => state.isLoading,

  hasError: (state: ContextStore): boolean => state.error !== null,

  getError: (state: ContextStore): string | null => state.error ?? null,

  getSSEStatus: (state: ContextStore): 'connected' | 'connecting' | 'disconnected' | 'error' =>
    state.sseStatus,

  getSSEError: (state: ContextStore): string | null => state.sseError ?? null,

  hasReceivedSnapshot: (state: ContextStore): boolean => state.hasReceivedSnapshot,

  isSSEConnected: (state: ContextStore): boolean => state.sseStatus === 'connected',

  isSSEConnecting: (state: ContextStore): boolean => state.sseStatus === 'connecting',

  isSSEDisconnected: (state: ContextStore): boolean => state.sseStatus === 'disconnected',

  hasSSEError: (state: ContextStore): boolean => state.sseStatus === 'error',
}
