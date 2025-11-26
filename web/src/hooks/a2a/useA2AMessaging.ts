/**
 * Hook for sending messages via A2A protocol.
 *
 * Handles text message sending and streaming with proper
 * error handling and state management.
 *
 * @module hooks/a2a/useA2AMessaging
 */

import { useState, useCallback } from 'react'
import { useContextStore } from '@/stores/context.store'
import { logger } from '@/lib/logger'
import type { A2AService } from '@/lib/a2a/client'
import type { Task, Message, TaskStatusUpdateEvent, TaskArtifactUpdateEvent } from '@a2a-js/sdk'

type A2AStreamEventData = Message | Task | TaskStatusUpdateEvent | TaskArtifactUpdateEvent

/**
 * Message sending state.
 */
interface MessagingState {
  /**
   * Whether a message is currently being sent
   */
  isSending: boolean

  /**
   * Error from last send operation
   */
  error: Error | null
}

/**
 * Messaging hook return value.
 */
interface UseA2AMessagingReturn extends MessagingState {
  /**
   * Send a text message to the agent
   */
  sendMessage: (text: string, files?: File[]) => Promise<Task | Message | null>

  /**
   * Stream messages from the agent
   */
  streamMessage: (text: string) => AsyncGenerator<A2AStreamEventData, void, unknown>

  /**
   * Retrieve a specific task by ID
   */
  getTask: (taskId: string) => Promise<Task | null>

  /**
   * Cancel a running task by ID
   */
  cancelTask: (taskId: string) => Promise<Task | null>

  /**
   * Clear current error state
   */
  clearError: () => void
}

/**
 * Handles A2A message sending in both regular and streaming modes.
 *
 * Provides methods for sending text messages, streaming responses,
 * and task management (get, cancel). Ensures client is ready before
 * each operation and manages error states.
 *
 * @param client - Initialized A2A client instance
 * @returns Message sending functions and state
 *
 * @example
 * ```typescript
 * function ChatInput({ client }) {
 *   const { sendMessage, isSending, error } = useA2AMessaging(client)
 *
 *   const handleSubmit = async (text: string) => {
 *     try {
 *       const response = await sendMessage(text)
 *       console.log('Message sent:', response)
 *     } catch (err) {
 *       console.error('Send failed:', err)
 *     }
 *   }
 *
 *   return (
 *     <div>
 *       <input onSubmit={handleSubmit} disabled={isSending} />
 *       {error && <div>{error.message}</div>}
 *     </div>
 *   )
 * }
 * ```
 *
 * @throws {Error} When client is not initialized
 * @throws {Error} When context is not ready
 * @throws {Error} When message send fails
 */
export function useA2AMessaging(client: A2AService | null): UseA2AMessagingReturn {
  const currentContextId = useContextStore((state) => state.currentContextId)
  const hasReceivedSnapshot = useContextStore((state) => state.hasReceivedSnapshot)
  const [state, setState] = useState<MessagingState>({
    isSending: false,
    error: null,
  })

  const clearError = useCallback(() => {
    setState((prev) => ({ ...prev, error: null }))
  }, [])

  /**
   * Validates that prerequisites for messaging are met.
   */
  const validateReadiness = useCallback((): boolean => {
    if (!client) {
      const err = new Error('Client not initialized')
      setState((prev) => ({ ...prev, error: err }))
      return false
    }

    if (!hasReceivedSnapshot) {
      const err = new Error('Cannot send message: Context not initialized. Please wait for contexts to load.')
      setState((prev) => ({ ...prev, error: err }))
      return false
    }

    return true
  }, [client, hasReceivedSnapshot])

  const sendMessage = useCallback(
    async (text: string, files?: File[]): Promise<Task | Message | null> => {
      if (!validateReadiness()) {
        return null
      }

      try {
        setState((prev) => ({ ...prev, isSending: true, error: null }))

        const response = await client!.sendMessage(text, files, currentContextId || undefined)
        logger.debug('A2A message sent', { length: text.length }, 'useA2AMessaging')

        return response
      } catch (err) {
        const errorToSet = err instanceof Error ? err : new Error('Failed to send message')
        setState((prev) => ({ ...prev, error: errorToSet }))
        logger.error('A2A message send failed', err, 'useA2AMessaging')
        throw errorToSet
      } finally {
        setState((prev) => ({ ...prev, isSending: false }))
      }
    },
    [client, currentContextId, validateReadiness]
  )

  const streamMessage = useCallback(
    async function* (text: string): AsyncGenerator<A2AStreamEventData, void, unknown> {
      if (!validateReadiness()) {
        return
      }

      try {
        setState((prev) => ({ ...prev, isSending: true, error: null }))

        for await (const event of client!.streamMessage(text, currentContextId || undefined)) {
          yield event
        }

        logger.debug('A2A streaming message completed', { length: text.length }, 'useA2AMessaging')
      } catch (err) {
        const errorToSet = err instanceof Error ? err : new Error('Failed to stream message')
        setState((prev) => ({ ...prev, error: errorToSet }))
        logger.error('A2A streaming failed', err, 'useA2AMessaging')
        throw errorToSet
      } finally {
        setState((prev) => ({ ...prev, isSending: false }))
      }
    },
    [client, currentContextId, validateReadiness]
  )

  const getTask = useCallback(
    async (taskId: string): Promise<Task | null> => {
      if (!validateReadiness()) {
        return null
      }

      try {
        setState((prev) => ({ ...prev, error: null }))
        const task = await client!.getTask(taskId)
        logger.debug('A2A task fetched', { taskId }, 'useA2AMessaging')
        return task
      } catch (err) {
        const errorToSet = err instanceof Error ? err : new Error('Failed to get task')
        setState((prev) => ({ ...prev, error: errorToSet }))
        logger.error('A2A get task failed', err, 'useA2AMessaging')
        throw errorToSet
      }
    },
    [client, validateReadiness]
  )

  const cancelTask = useCallback(
    async (taskId: string): Promise<Task | null> => {
      if (!validateReadiness()) {
        return null
      }

      try {
        setState((prev) => ({ ...prev, error: null }))
        const task = await client!.cancelTask(taskId)
        logger.debug('A2A task cancelled', { taskId }, 'useA2AMessaging')
        return task
      } catch (err) {
        const errorToSet = err instanceof Error ? err : new Error('Failed to cancel task')
        setState((prev) => ({ ...prev, error: errorToSet }))
        logger.error('A2A cancel task failed', err, 'useA2AMessaging')
        throw errorToSet
      }
    },
    [client, validateReadiness]
  )

  return {
    ...state,
    sendMessage,
    streamMessage,
    getTask,
    cancelTask,
    clearError,
  }
}
