import { useCallback } from 'react'
import { A2AService } from '@/lib/a2a/client'
import { useContextStore, CONTEXT_STATE } from '@/stores/context.store'
import type { Task, Message, TaskStatusUpdateEvent, TaskArtifactUpdateEvent } from '@a2a-js/sdk'

type A2AStreamEventData = Message | Task | TaskStatusUpdateEvent | TaskArtifactUpdateEvent

/**
 * Hook for A2A message and task operations.
 *
 * Handles:
 * - Sending text messages with optional files
 * - Streaming messages from agent
 * - Task retrieval and cancellation
 * - Context state validation
 * - Error handling for all operations
 *
 * @param client - The A2A client instance
 * @param ensureClientReady - Function to validate client state
 * @param onError - Callback to set error state
 * @returns {Object} Message and task operations
 *
 * @example
 * ```typescript
 * const { sendMessage, streamMessage, getTask } = useA2AMessageOperations(
 *   client,
 *   ensureClientReady,
 *   setError
 * )
 *
 * const task = await sendMessage('Hello')
 * for await (const event of streamMessage('Stream me')) {
 *   handleEvent(event)
 * }
 * ```
 */
export function useA2AMessageOperations(
  client: A2AService | null,
  ensureClientReady: () => Promise<boolean>,
  onError: (error: Error | null) => void
) {
  const currentContextId = useContextStore((state) => state.currentContextId)
  const hasReceivedSnapshot = useContextStore((state) => state.hasReceivedSnapshot)

  const sendMessage = useCallback(
    async (text: string, files?: File[]): Promise<Task | Message | null> => {
      const isReady = await ensureClientReady()
      if (!isReady) {
        return null
      }

      if (!hasReceivedSnapshot) {
        const err = new Error('Cannot send message: Context not initialized. Please wait for contexts to load.')
        onError(err)
        throw err
      }

      try {
        onError(null)
        const contextId = currentContextId === CONTEXT_STATE.LOADING ? undefined : currentContextId
        const response = await client!.sendMessage(text, files, contextId || undefined)
        return response
      } catch (err) {
        const errorToSet = err instanceof Error ? err : new Error(typeof err === 'string' ? err : 'Failed to send message')
        onError(errorToSet)
        throw errorToSet
      }
    },
    [client, currentContextId, hasReceivedSnapshot, ensureClientReady, onError]
  )

  const streamMessage = useCallback(
    async function* (text: string, clientMessageId?: string): AsyncGenerator<A2AStreamEventData, void, unknown> {
      const isReady = await ensureClientReady()
      if (!isReady) {
        return
      }

      if (!hasReceivedSnapshot) {
        const err = new Error('Cannot send message: Context not initialized. Please wait for contexts to load.')
        onError(err)
        throw err
      }

      try {
        onError(null)
        const contextId = currentContextId === CONTEXT_STATE.LOADING ? undefined : currentContextId
        for await (const event of client!.streamMessage(text, contextId || undefined, clientMessageId)) {
          yield event
        }
      } catch (err) {
        const errorToSet = err instanceof Error ? err : new Error(typeof err === 'string' ? err : 'Failed to stream message')
        onError(errorToSet)
        throw errorToSet
      }
    },
    [client, currentContextId, hasReceivedSnapshot, ensureClientReady, onError]
  )

  const getTask = useCallback(
    async (taskId: string): Promise<Task | null> => {
      const isReady = await ensureClientReady()
      if (!isReady) {
        return null
      }

      try {
        onError(null)
        return await client!.getTask(taskId)
      } catch (err) {
        const errorToSet = err instanceof Error ? err : new Error(typeof err === 'string' ? err : 'Failed to get task')
        onError(errorToSet)
        throw errorToSet
      }
    },
    [client, ensureClientReady, onError]
  )

  const cancelTask = useCallback(
    async (taskId: string): Promise<Task | null> => {
      const isReady = await ensureClientReady()
      if (!isReady) {
        return null
      }

      try {
        onError(null)
        return await client!.cancelTask(taskId)
      } catch (err) {
        const errorToSet = err instanceof Error ? err : new Error(typeof err === 'string' ? err : 'Failed to cancel task')
        onError(errorToSet)
        throw errorToSet
      }
    },
    [client, ensureClientReady, onError]
  )

  return {
    sendMessage,
    streamMessage,
    getTask,
    cancelTask
  }
}
