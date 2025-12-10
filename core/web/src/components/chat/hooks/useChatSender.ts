/**
 * Hook for sending chat messages.
 *
 * Sends messages to the backend. State updates (user message appearing,
 * assistant response, execution steps) all come via SSE events:
 * - task_created: User message appears
 * - execution_step: Progress tracking
 * - task_status_changed: Assistant response
 *
 * No optimistic updates or streaming text accumulation - backend is single source of truth.
 *
 * @module chat/hooks/useChatSender
 */

import { useState, useCallback } from 'react'
import { useA2AClient } from '@/hooks/useA2AClient'
import { useTaskStore } from '@/stores/task.store'
import { useArtifactStore } from '@/stores/artifact.store'
import { useUIStateStore } from '@/stores/ui-state.store'
import { isTaskEvent, isStatusUpdateEvent } from '../helpers/typeGuards'
import type { Task } from '@/types/task'
import type { Artifact } from '@/types/artifact'
import type { Task as A2ATask } from '@a2a-js/sdk'
import { toTask } from '@/types/task'
import { toArtifact } from '@/types/artifact'

interface UseChatSenderReturn {
  sendMessage: (text: string, files?: File[]) => Promise<void>
  isSending: boolean
  error: string | null
  clearError: () => void
}

/**
 * Handles sending chat messages.
 *
 * The message sending triggers backend processing which broadcasts events via SSE.
 * UI updates happen automatically as the frontend receives these events.
 *
 * @returns Message sending functions and state
 */
export function useChatSender(): UseChatSenderReturn {
  const { streamMessage, sendMessage: sendMessageApi } = useA2AClient()

  const [isSending, setIsSending] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const clearError = useCallback(() => {
    setError(null)
  }, [])

  const sendMessage = useCallback(
    async (text: string, files?: File[]) => {
      try {
        setIsSending(true)
        setError(null)

        if (!files?.length && streamMessage) {
          // Use streaming endpoint - consume stream to completion
          // SSE events will update the UI (task_created, execution_step, task_status_changed)
          try {
            for await (const event of streamMessage(text)) {
              // Handle status-update events (for failed/canceled states)
              if (isStatusUpdateEvent(event)) {
                const state = event.status.state
                if (state === 'failed' || state === 'rejected' || state === 'canceled') {
                  useUIStateStore.getState().clearStepsByTask(event.taskId)
                  useUIStateStore.getState().setStreaming(null)

                  // Update task store with failed status
                  const existingTask = useTaskStore.getState().byId[event.taskId]
                  if (existingTask) {
                    useTaskStore.getState().updateTask({
                      ...existingTask,
                      status: {
                        state: state as 'failed' | 'rejected' | 'canceled',
                        message: event.status.message ? {
                          role: 'agent',
                          parts: [{ kind: 'text' as const, text: event.status.message }],
                          messageId: '',
                          kind: 'message',
                          contextId: event.contextId || existingTask.contextId || '',
                        } : undefined,
                        timestamp: new Date().toISOString(),
                      },
                    })
                  }
                }
                continue
              }

              // Handle task events to update store (backup to SSE)
              if (isTaskEvent(event)) {
                const rawTask = event as A2ATask

                let validatedTask: Task
                try {
                  validatedTask = toTask(rawTask)
                } catch (e) {
                  continue
                }

                useTaskStore.getState().updateTask(validatedTask)

                // Clear streaming state when task reaches terminal state (backup to SSE handler)
                const state = validatedTask.status?.state
                if (state === 'completed' || state === 'failed' || state === 'rejected' || state === 'canceled') {
                  useUIStateStore.getState().clearStepsByTask(validatedTask.id)
                  useUIStateStore.getState().setStreaming(null)
                }

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
          } catch (streamError: unknown) {
            // Clear streaming state on any stream error
            useUIStateStore.getState().setStreaming(null)

            const errorMessage = streamError instanceof Error ? streamError.message : String(streamError)
            const isPermissionError =
              errorMessage.includes('401') ||
              errorMessage.includes('403') ||
              errorMessage.includes('Unauthorized') ||
              errorMessage.includes('Forbidden')

            if (isPermissionError) {
              setError('Permission denied. You may not have sufficient permissions to access this agent.')
              return
            }

            throw streamError
          }
        } else {
          // Handle non-streaming message
          await sendMessageApi?.(text, files)
        }
      } catch (err: unknown) {
        // Clear streaming state on any error
        useUIStateStore.getState().setStreaming(null)

        const errorMessage = err instanceof Error ? err.message : String(err)
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
      } finally {
        setIsSending(false)
      }
    },
    [streamMessage, sendMessageApi]
  )

  return { sendMessage, isSending, error, clearError }
}
