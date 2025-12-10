import { create } from 'zustand'
import { contextsService } from '@/services/contexts.service'
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

interface ContextEventBase {
  type: string
  context_id?: string
  timestamp: string
}

interface ContextCreatedEvent extends ContextEventBase {
  type: 'context_created'
  context: {
    context_id: string
    name: string
    created_at: string
    updated_at: string
  }
}

interface ContextUpdatedEvent extends ContextEventBase {
  type: 'context_updated'
  name: string
}

interface ContextDeletedEvent extends ContextEventBase {
  type: 'context_deleted'
}

interface CurrentAgentEvent extends ContextEventBase {
  type: 'current_agent'
  agent_name: string | null
}

type ContextStateEvent =
  | ContextCreatedEvent
  | ContextUpdatedEvent
  | ContextDeletedEvent
  | CurrentAgentEvent
  | (ContextEventBase & { type: string })

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
    switch (event.type) {
      case 'context_created': {
        const createdEvent = event as ContextCreatedEvent
        if (!createdEvent.context?.context_id) return

        const newConversation: Conversation = {
          id: createdEvent.context.context_id,
          name: createdEvent.context.name,
          createdAt: new Date(createdEvent.context.created_at),
          updatedAt: new Date(createdEvent.context.updated_at),
          messageCount: 0,
        }
        set((state) => {
          const updated = new Map(state.conversations)
          updated.set(createdEvent.context.context_id, newConversation)
          return {
            conversations: updated,
            currentContextId: state.currentContextId === CONTEXT_STATE.LOADING
              ? createdEvent.context_id
              : state.currentContextId,
          }
        })
        break
      }

      case 'context_updated': {
        const updatedEvent = event as ContextUpdatedEvent
        if (!updatedEvent.context_id) return

        const contextId = updatedEvent.context_id
        set((state) => {
          const updated = new Map(state.conversations)
          const conv = updated.get(contextId)
          if (conv) {
            updated.set(contextId, { ...conv, name: updatedEvent.name, updatedAt: new Date(updatedEvent.timestamp) })
          }
          return { conversations: updated }
        })
        break
      }

      case 'context_deleted': {
        const deletedEvent = event as ContextDeletedEvent
        if (!deletedEvent.context_id) return

        const contextId = deletedEvent.context_id
        set((state) => {
          const updated = new Map(state.conversations)
          updated.delete(contextId)
          const newCurrentId = state.currentContextId === contextId
            ? Array.from(updated.keys())[0] || CONTEXT_STATE.LOADING
            : state.currentContextId
          return { conversations: updated, currentContextId: newCurrentId }
        })
        break
      }

      case 'current_agent': {
        const agentEvent = event as CurrentAgentEvent
        if (!agentEvent.context_id) return

        const contextId = agentEvent.context_id
        const agentName = agentEvent.agent_name

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
          const matchingAgent = agentStore.agents.find(
            agent => agent.name.toLowerCase() === agentName.toLowerCase()
          )
          if (matchingAgent && get().currentContextId === contextId) {
            agentStore.selectAgent(matchingAgent.url, matchingAgent)
          }
        }
        break
      }
    }
  },
}))
