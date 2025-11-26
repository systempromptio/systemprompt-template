import { useEffect, useRef, useState, useMemo } from 'react'
import { useTaskStore } from '@/stores/task.store'
import { useContextStore, CONTEXT_STATE } from '@/stores/context.store'
import { useArtifactStore } from '@/stores/artifact.store'
import { useAuthStore } from '@/stores/auth.store'
import { tasksService } from '@/services/tasks.service'
import { mapTasksToChatMessages } from '@/utils/message-mapper'
import { toArtifact } from '@/types/artifact'
import { extractAndStoreSkill } from '@/lib/utils/extractArtifactSkills'
import type { ChatMessage } from '@/stores/chat.store'
import type { Task } from '@/types/task'

interface UseChatMessageLoaderReturn {
  messages: ChatMessage[]
  isLoading: boolean
  optimisticMessages: ChatMessage[]
  setOptimisticMessages: React.Dispatch<React.SetStateAction<ChatMessage[]>>
}

export function useChatMessageLoader(): UseChatMessageLoaderReturn {
  const currentContextId = useContextStore((state) => state.currentContextId)
  const updateMessageCount = useContextStore((state) => state.updateMessageCount)
  const isLoadingContexts = useContextStore((state) => state.isLoading)
  const getAuthHeader = useAuthStore((state) => state.getAuthHeader)

  const byContext = useTaskStore((state) => state.byContext)
  const byId = useTaskStore((state) => state.byId)

  const [optimisticMessages, setOptimisticMessages] = useState<ChatMessage[]>([])
  const [isLoadingMessages, setIsLoadingMessages] = useState(false)
  const abortControllerRef = useRef<AbortController | null>(null)

  // Derived messages from task store
  const derivedMessages = useMemo(() => {
    if (!currentContextId) return []

    const taskIds = byContext[currentContextId] || []
    const tasksForContext = taskIds
      .map((id: string) => byId[id])
      .filter((task): task is Task => task !== undefined)

    const messages = mapTasksToChatMessages(tasksForContext, currentContextId)
    return messages
  }, [byContext, byId, currentContextId])

  // Merge derived and optimistic messages with direct ID correlation
  const messages = useMemo(() => {
    const now = Date.now()
    const OPTIMISTIC_TTL_MS = 300000 // 5 minutes - allows for long-running AI responses

    // Map backend messages by their clientMessageId for quick lookup (for assistant messages)
    const backendByClientId = new Map<string, ChatMessage>()
    derivedMessages.forEach(dm => {
      const clientId = dm.metadata?.clientMessageId
      if (clientId && typeof clientId === 'string') {
        backendByClientId.set(clientId, dm)
      }
    })

    // Map backend user messages by content for deduplication
    const backendUserMessageContents = new Set<string>()
    derivedMessages.forEach(dm => {
      if (dm.role === 'user' && dm.content) {
        backendUserMessageContents.add(dm.content.trim())
      }
    })

    // Filter optimistic messages: keep only those without a matching backend message
    const validOptimistic = optimisticMessages.filter(m => {
      const belongsToCurrentContext = m.contextId === currentContextId
      if (!belongsToCurrentContext) return false

      // Check if backend has a message with this optimistic ID as clientMessageId (for assistant messages)
      const hasBackendMatch = backendByClientId.has(m.id)
      if (hasBackendMatch) {
        // Backend message exists, don't show optimistic
        return false
      }

      // For user messages, check if backend has a user message with the same content
      if (m.role === 'user' && m.content) {
        const hasBackendUserMessage = backendUserMessageContents.has(m.content.trim())
        if (hasBackendUserMessage) {
          // Backend user message with same content exists, don't show optimistic
          return false
        }
      }

      // Streaming messages are never filtered by TTL
      const isStreaming = m.isStreaming === true
      if (isStreaming) return true

      // Non-streaming messages kept only if recent
      const msgTimestamp = m.timestamp instanceof Date ? m.timestamp : new Date(m.timestamp)
      const isRecent = (now - msgTimestamp.getTime()) < OPTIMISTIC_TTL_MS
      return isRecent
    })

    const merged = [...derivedMessages, ...validOptimistic]
    return merged
  }, [derivedMessages, optimisticMessages, currentContextId])

  // Listen to artifact store changes for SSE-triggered refetch
  useEffect(() => {
    if (!currentContextId) return

    const unsubscribe = useArtifactStore.subscribe((state, prevState) => {
      if (!currentContextId) return

      const currentCtxArtifacts = Object.values(state.byId).filter(
        a => a.metadata.context_id === currentContextId
      )
      const prevCtxArtifacts = Object.values(prevState.byId).filter(
        a => a.metadata.context_id === currentContextId
      )

      if (currentCtxArtifacts.length > prevCtxArtifacts.length) {
        updateMessageCount(currentContextId)
      }
    })

    return () => {
      unsubscribe()
    }
  }, [currentContextId, updateMessageCount])

  // Load messages from backend
  useEffect(() => {
    const authHeader = getAuthHeader()

    if (abortControllerRef.current) {
      abortControllerRef.current.abort()
    }

    if (!currentContextId || !authHeader || currentContextId === CONTEXT_STATE.LOADING) {
      return
    }

    const controller = new AbortController()
    abortControllerRef.current = controller

    setIsLoadingMessages(true)

    tasksService.listTasksByContext(currentContextId, authHeader)
      .then(({ tasks, error }) => {
        if (controller.signal.aborted) {
          return
        }

        if (error) {
          console.error('Failed to load messages:', error)
        } else if (tasks) {
          tasks.forEach(task => {
            useTaskStore.getState().updateTask(task)
          })

          tasks.forEach((task) => {
            if (task.artifacts && task.artifacts.length > 0) {
              task.artifacts.forEach((artifact) => {
                try {
                  const validated = toArtifact(artifact)
                  useArtifactStore.getState().addArtifact(
                    validated,
                    task.id,
                    currentContextId
                  )
                  // Extract skill metadata from artifact when loading messages
                  extractAndStoreSkill(validated, currentContextId, task.id)
                } catch (e) {
                  // Skip invalid artifact
                }
              })
            }
          })
        }
      })
      .catch(err => {
        if (err.name === 'AbortError' || controller.signal.aborted) {
          return
        }
        console.error('Error loading messages:', err)
      })
      .finally(() => {
        if (!controller.signal.aborted) {
          setIsLoadingMessages(false)
        }
      })

    return () => {
      if (abortControllerRef.current) {
        abortControllerRef.current.abort()
      }
    }
  }, [currentContextId, getAuthHeader])

  return {
    messages,
    isLoading: isLoadingMessages || isLoadingContexts,
    optimisticMessages,
    setOptimisticMessages,
  }
}
