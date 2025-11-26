/**
 * Hook for sending chat messages with streaming support.
 *
 * Handles both regular text messages and streaming messages with
 * proper error handling, optimistic updates, and artifact finalization.
 *
 * @module chat/hooks/useChatSender
 */

import { useState, useCallback } from 'react'
import { useA2AClient } from '@/hooks/useA2AClient'
import { useTaskStore } from '@/stores/task.store'
import { useArtifactStore } from '@/stores/artifact.store'
import { useStreamProcessor } from './useStreamProcessor'
import { isTaskEvent } from '../helpers/typeGuards'
import type { Task } from '@/types/task'
import type { Artifact } from '@/types/artifact'
import type { Task as A2ATask } from '@a2a-js/sdk'
import { toTask } from '@/types/task'
import { toArtifact } from '@/types/artifact'
import type { ChatMessage } from '@/stores/chat.store'

/**
 * Chat sender hook parameters.
 */
interface UseChatSenderParams {
  /**
   * Function to update optimistic messages
   */
  setOptimisticMessages: React.Dispatch<React.SetStateAction<ChatMessage[]>>
}

/**
 * Chat sender hook return value.
 */
interface UseChatSenderReturn {
  /**
   * Sends a chat message (handles both text and streaming)
   */
  sendMessage: (text: string, files?: File[], optimisticMessageId?: string) => Promise<void>

  /**
   * Whether a message is currently being sent
   */
  isSending: boolean

  /**
   * Error from the last send operation, if any
   */
  error: string | null

  /**
   * Clears the error message
   */
  clearError: () => void
}

/**
 * Handles sending chat messages in both regular and streaming modes.
 *
 * Features:
 * - Optimistic UI updates
 * - Streaming with real-time artifact accumulation
 * - Task handling from stream events
 * - Error handling and recovery
 *
 * @returns Message sending functions and state
 *
 * @example
 * ```typescript
 * function ChatInterface() {
 *   const [optimisticMessages, setOptimisticMessages] = useState<ChatMessage[]>([])
 *   const { sendMessage, isSending, error } = useChatSender({ setOptimisticMessages })
 *
 *   const handleSubmit = async (text: string) => {
 *     const messageId = crypto.randomUUID()
 *     await sendMessage(text, undefined, messageId)
 *   }
 *
 *   return (
 *     <div>
 *       {error && <div className="error">{error}</div>}
 *       <input disabled={isSending} />
 *       <button onClick={() => handleSubmit(text)} disabled={isSending}>Send</button>
 *     </div>
 *   )
 * }
 * ```
 */
export function useChatSender({ setOptimisticMessages }: UseChatSenderParams): UseChatSenderReturn {
  const { streamMessage, sendMessage: sendMessageApi } = useA2AClient()
  const { processEvent, reset, setStreaming, finalizeArtifacts: finalizeStreamState, getStreamState } = useStreamProcessor()

  const [isSending, setIsSending] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const clearError = useCallback(() => {
    setError(null)
  }, [])

  const sendMessage = useCallback(
    async (text: string, files?: File[], optimisticMessageId?: string) => {
      try {
        setIsSending(true)
        setError(null)

        // Reset streaming state
        reset()
        setStreaming(true)

        if (!files?.length && streamMessage) {
          let currentOptimisticId = optimisticMessageId

          try {
            for await (const event of streamMessage(text, optimisticMessageId)) {
              // Process message events
              processEvent(event)

              // Note: We send the optimistic message ID to the backend via metadata.clientMessageId
              // The backend stores it and returns it, allowing us to correlate optimistic messages
              // with backend messages for seamless reconciliation.

              // Update optimistic message with stream content
              if (currentOptimisticId) {
                const streamState = getStreamState()
                setOptimisticMessages((prev) =>
                  prev.map((m) =>
                    m.id === currentOptimisticId
                      ? {
                          ...m,
                          content: streamState.text,
                          parts: [{ kind: 'text', text: streamState.text }],
                          artifacts: Array.from(streamState.artifacts.values()),
                          streamingArtifacts: streamState.streamingArtifactsState,
                          isStreaming: true,
                        }
                      : m
                  )
                )
              }

              // Handle task events
              if (isTaskEvent(event)) {
                const rawTask = event as A2ATask

                let validatedTask: Task
                try {
                  validatedTask = toTask(rawTask)
                } catch (e) {
                  continue
                }

                useTaskStore.getState().updateTask(validatedTask)

                // Process task artifacts
                if (validatedTask.artifacts && validatedTask.artifacts.length > 0) {
                  const validatedArtifacts = validatedTask.artifacts
                    .map((a) => {
                      try {
                        return toArtifact(a)
                      } catch (e) {
                        return null
                      }
                    })
                    .filter((a): a is Artifact => a !== null)

                  validatedArtifacts.forEach((artifact) => {
                    useArtifactStore.getState().addArtifact(
                      artifact,
                      validatedTask.id,
                      validatedTask.contextId || ''
                    )
                  })
                }
              }
            }

            setStreaming(false)
            finalizeStreamState()

            // Clear streaming flag from assistant message now that streaming is complete
            // (user message will be removed by deduplication when backend message arrives)
            if (currentOptimisticId) {
              setOptimisticMessages((prev) =>
                prev.map((m) =>
                  m.id === currentOptimisticId
                    ? { ...m, isStreaming: false }
                    : m
                )
              )
            }
          } catch (streamError: any) {
            const errorMessage = streamError?.message || String(streamError)
            const isPermissionError =
              errorMessage.includes('401') ||
              errorMessage.includes('403') ||
              errorMessage.includes('Unauthorized') ||
              errorMessage.includes('Forbidden')

            if (isPermissionError) {
              setError('Permission denied. You may not have sufficient permissions to access this agent.')
              // Remove the streaming flag from optimistic message on error
              if (currentOptimisticId) {
                setOptimisticMessages((prev) =>
                  prev.map((m) =>
                    m.id === currentOptimisticId ? { ...m, isStreaming: false } : m
                  )
                )
              }
              return
            }

            throw streamError
          }
        } else {
          // Handle non-streaming message
          await sendMessageApi?.(text, files)
        }
      } catch (err: any) {
        const errorMessage = err?.message || String(err)
        const isPermissionError =
          errorMessage.includes('401') ||
          errorMessage.includes('403') ||
          errorMessage.includes('Unauthorized') ||
          errorMessage.includes('Forbidden')

        if (isPermissionError) {
          setError('Permission denied. You may not have sufficient permissions to access this agent.')
        } else {
          setError(err instanceof Error ? err.message : 'Failed to send message')
        }

        // Remove the streaming flag from optimistic message on error
        if (optimisticMessageId) {
          setOptimisticMessages((prev) =>
            prev.map((m) =>
              m.id === optimisticMessageId ? { ...m, isStreaming: false } : m
            )
          )
        }
      } finally {
        setIsSending(false)
        setStreaming(false)
      }
    },
    [streamMessage, sendMessageApi, reset, setStreaming, processEvent, finalizeStreamState, getStreamState, setOptimisticMessages]
  )

  return { sendMessage, isSending, error, clearError }
}
