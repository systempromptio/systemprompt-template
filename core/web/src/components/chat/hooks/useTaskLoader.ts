import { useEffect, useRef, useState, useMemo } from 'react'
import { useTaskStore } from '@/stores/task.store'
import { useContextStore, CONTEXT_STATE } from '@/stores/context.store'
import { useArtifactStore } from '@/stores/artifact.store'
import { useUIStateStore } from '@/stores/ui-state.store'
import { useAuthStore } from '@/stores/auth.store'
import { tasksService } from '@/services/tasks.service'
import { toArtifact } from '@/types/artifact'
import { extractAndStoreSkill } from '@/lib/utils/extractArtifactSkills'
import { logger } from '@/lib/logger'
import type { Task } from '@/types/task'

interface UseTaskLoaderReturn {
  tasks: Task[]
  isLoading: boolean
  contextId: string | null
}

/**
 * Loads tasks for current context.
 *
 * Tasks come from two sources:
 * 1. SSE events (task_created, task_status_changed) - real-time updates
 * 2. API fetch on context change - initial load and catch-up
 *
 * No transformation - returns Task[] directly from store.
 */
export function useTaskLoader(): UseTaskLoaderReturn {
  const currentContextId = useContextStore((s) => s.currentContextId)
  const updateMessageCount = useContextStore((s) => s.updateMessageCount)
  const isLoadingContexts = useContextStore((s) => s.isLoading)
  const getAuthHeader = useAuthStore((s) => s.getAuthHeader)

  const byContext = useTaskStore((s) => s.byContext)
  const byId = useTaskStore((s) => s.byId)

  const [isLoadingTasks, setIsLoadingTasks] = useState(false)
  const abortControllerRef = useRef<AbortController | null>(null)

  const tasks = useMemo(() => {
    if (!currentContextId) return []

    const taskIds = byContext[currentContextId] || []
    return taskIds
      .map((id) => byId[id])
      .filter((task): task is Task => task !== undefined)
      // Filter out direct MCP executions - only show agent_message tasks in conversation
      .filter((task) => task.metadata?.task_type !== 'mcp_execution')
      .sort((a, b) => {
        const aTime = new Date(a.metadata?.created_at || 0).getTime()
        const bTime = new Date(b.metadata?.created_at || 0).getTime()
        return aTime - bTime
      })
  }, [byContext, byId, currentContextId])

  useEffect(() => {
    if (!currentContextId) return

    const unsubscribe = useArtifactStore.subscribe((state, prevState) => {
      const currentCtxArtifacts = Object.values(state.byId).filter(
        (a) => a.metadata.context_id === currentContextId
      )
      const prevCtxArtifacts = Object.values(prevState.byId).filter(
        (a) => a.metadata.context_id === currentContextId
      )

      if (currentCtxArtifacts.length > prevCtxArtifacts.length) {
        updateMessageCount(currentContextId)
      }
    })

    return () => unsubscribe()
  }, [currentContextId, updateMessageCount])

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

    setIsLoadingTasks(true)

    tasksService
      .listTasksByContext(currentContextId, authHeader)
      .then(({ tasks: fetchedTasks, error }) => {
        if (controller.signal.aborted) return

        if (error) {
          logger.error('Failed to load tasks', error, 'useTaskLoader')
          return
        }

        if (!fetchedTasks) return

        fetchedTasks.forEach((task) => {
          useTaskStore.getState().updateTask(task)

          const executionSteps = task.metadata?.executionSteps
          if (executionSteps && Array.isArray(executionSteps) && executionSteps.length > 0) {
            // Ensure each step has the taskId set (backend may not include it)
            const stepsWithTaskId = executionSteps.map((step) => ({
              ...step,
              taskId: step.taskId || task.id,
            }))
            useUIStateStore.getState().addSteps(stepsWithTaskId, currentContextId)
          }

          if (task.artifacts && task.artifacts.length > 0) {
            task.artifacts.forEach((artifact) => {
              try {
                const validated = toArtifact(artifact)
                useArtifactStore.getState().addArtifact(
                  validated,
                  task.id,
                  currentContextId
                )
                extractAndStoreSkill(validated, currentContextId, task.id)
              } catch (e) {
                logger.warn('Skipping invalid artifact', e, 'useTaskLoader')
              }
            })
          }
        })
      })
      .catch((err) => {
        if (err.name === 'AbortError' || controller.signal.aborted) return
        logger.error('Error loading tasks', err, 'useTaskLoader')
      })
      .finally(() => {
        if (!controller.signal.aborted) {
          setIsLoadingTasks(false)
        }
      })

    return () => {
      if (abortControllerRef.current) {
        abortControllerRef.current.abort()
      }
    }
  }, [currentContextId, getAuthHeader])

  return {
    tasks,
    isLoading: isLoadingTasks || isLoadingContexts,
    contextId: currentContextId,
  }
}
