/**
 * Stream Event Handler Utilities
 *
 * Extracted handlers for different SSE event types.
 * Each handler is responsible for a single event type.
 *
 * Architecture: SSE Event -> Validate -> Store -> Components
 * Validation happens at the boundary. After validation, types are trusted.
 */

import { useTaskStore } from '@/stores/task.store'
import { useArtifactStore } from '@/stores/artifact.store'
import { useContextStore } from '@/stores/context.store'
import { useAuthStore } from '@/stores/auth.store'
import { useUIStateStore } from '@/stores/ui-state.store'
import type { Task as A2ATask } from '@a2a-js/sdk'
import type { BroadcastEvent } from '@/types/sse'
import { hasSkillInData } from '@/types/sse'
import type { ExecutionStep } from '@/types/execution'
import { logger } from '@/lib/logger'
import { extractAndStoreSkill } from '@/lib/utils/extractArtifactSkills'
import { hasTaskInData, getStatusMessageId, isPlainObject } from '@/utils/type-guards'
import {
  EventValidationError,
  validateTaskEvent,
  validateArtifactEvent,
  validateArtifactsArray,
  validateContextId,
  hasArtifactsInData,
  hasExecutionStepsInData,
} from './eventValidators'

interface ContextStatsItem {
  context_id: string
  message_count: number
}

function isContextStatsItem(value: unknown): value is ContextStatsItem {
  if (!isPlainObject(value)) return false
  return (
    typeof value.context_id === 'string' &&
    typeof value.message_count === 'number'
  )
}

/**
 * Handle snapshot event - loads all contexts at once
 */
export function handleSnapshotEvent(data: string): void {
  try {
    const parsed = JSON.parse(data)
    const contexts = parsed.contexts || parsed.data?.contexts
    logger.debug('Received snapshot', { contextCount: contexts?.length }, 'streamEventHandlers')

    if (contexts && Array.isArray(contexts)) {
      useContextStore.getState().handleSnapshot(contexts)
    }
  } catch (error) {
    logger.error('Failed to parse snapshot event', error, 'streamEventHandlers')
  }
}

/**
 * Handle context stats event - updates message counts
 */
export function handleContextStatsEvent(data: string): void {
  try {
    const parsed = JSON.parse(data)
    const contexts = parsed.contexts || parsed.data?.contexts
    logger.debug('Received context stats', { contextCount: contexts?.length }, 'streamEventHandlers')

    if (contexts && Array.isArray(contexts)) {
      const store = useContextStore.getState()
      contexts.forEach((ctx: unknown) => {
        if (isContextStatsItem(ctx)) {
          const existing = store.conversations.get(ctx.context_id)
          if (existing) {
            const updated = new Map(store.conversations)
            updated.set(ctx.context_id, {
              ...existing,
              messageCount: ctx.message_count
            })
            useContextStore.setState({ conversations: updated })
          }
        }
      })
    }
  } catch (error) {
    logger.error('Failed to parse context stats event', error, 'streamEventHandlers')
  }
}

/**
 * Handle task_created event - new task with user message
 * This is the authoritative source for user messages (no optimistic state needed)
 */
export function handleTaskCreatedEvent(event: BroadcastEvent): void {
  console.log('%c[TASK_CREATED] Raw event received', 'background: #00ff00; color: black; font-weight: bold;', {
    timestamp: new Date().toISOString(),
    eventType: event.event_type,
    contextId: event.context_id,
    rawData: event.data
  })

  try {
    const validatedTask = validateTaskEvent(event)

    console.log('%c[TASK_CREATED] Validated task', 'background: #00ff00; color: black;', {
      taskId: validatedTask.id,
      status: validatedTask.status?.state,
      historyLength: validatedTask.history?.length || 0,
      historyRoles: validatedTask.history?.map(m => ({
        messageId: m.messageId,
        role: m.role,
        partsCount: m.parts?.length,
        textPreview: m.parts?.[0]?.kind === 'text' ? (m.parts[0] as { text?: string }).text?.substring(0, 50) : 'N/A'
      }))
    })

    logger.debug('Task created', { taskId: validatedTask.id, contextId: validatedTask.contextId }, 'streamEventHandlers')
    useTaskStore.getState().updateTask(validatedTask)

    useUIStateStore.getState().setStreaming(validatedTask.id)

    if (event.context_id) {
      useContextStore.getState().updateMessageCount(event.context_id)
    }
  } catch (error) {
    if (error instanceof EventValidationError) {
      logger.error('Invalid task_created event', {
        eventType: error.eventType,
        message: error.message,
        receivedKeys: Object.keys(error.rawEvent.data || {}),
      }, 'streamEventHandlers')
      return
    }
    throw error
  }
}

/**
 * Handle task event - updates task store
 */
export function handleTaskEvent(event: BroadcastEvent): void {
  if (hasTaskInData(event.data)) {
    try {
      const validatedTask = validateTaskEvent(event)
      logger.debug('Updating task store', { taskId: validatedTask.id }, 'streamEventHandlers')
      useTaskStore.getState().updateTask(validatedTask)

      const state = validatedTask.status?.state

      if (state === 'input-required') {
        useUIStateStore.getState().registerInputRequest({
          taskId: validatedTask.id,
          messageId: getStatusMessageId(validatedTask) ?? '',
          message: validatedTask.status?.message,
          timestamp: new Date(),
        })
      }

      if (state === 'auth-required') {
        useUIStateStore.getState().registerAuthRequest({
          taskId: validatedTask.id,
          messageId: getStatusMessageId(validatedTask) ?? '',
          message: validatedTask.status?.message,
          timestamp: new Date(),
        })
      }

      if (state === 'working') {
        useUIStateStore.getState().setStreaming(validatedTask.id)
        useUIStateStore.getState().resolveInputRequest(validatedTask.id)
        useUIStateStore.getState().resolveAuthRequest(validatedTask.id)
      }

      if (state === 'completed') {
        console.log('%c[TASK_STATUS_CHANGED] Task completed - clearing streaming', 'background: #ff9900; color: black; font-weight: bold;', {
          timestamp: new Date().toISOString(),
          taskId: validatedTask.id,
          currentStreamingTaskId: useUIStateStore.getState().activeStreamingTaskId,
          stepsBeingCleared: useUIStateStore.getState().stepIdsByTask[validatedTask.id]?.length || 0
        })

        useUIStateStore.getState().clearStepsByTask(validatedTask.id)
        useUIStateStore.getState().setStreaming(null)
        useUIStateStore.getState().resolveInputRequest(validatedTask.id)
        useUIStateStore.getState().resolveAuthRequest(validatedTask.id)
      }

      if (state === 'failed' || state === 'rejected' || state === 'canceled') {
        console.log('%c[TASK_STATUS_CHANGED] Task failed/rejected/canceled - clearing streaming', 'background: #ff0000; color: white; font-weight: bold;', {
          timestamp: new Date().toISOString(),
          taskId: validatedTask.id,
          state,
          errorMessage: validatedTask.status?.message,
          currentStreamingTaskId: useUIStateStore.getState().activeStreamingTaskId,
          stepsBeingCleared: useUIStateStore.getState().stepIdsByTask[validatedTask.id]?.length || 0
        })

        useUIStateStore.getState().clearStepsByTask(validatedTask.id)
        useUIStateStore.getState().setStreaming(null)
        useUIStateStore.getState().resolveInputRequest(validatedTask.id)
        useUIStateStore.getState().resolveAuthRequest(validatedTask.id)
      }

      const executionSteps = validatedTask.metadata?.executionSteps
      if (executionSteps && Array.isArray(executionSteps) && executionSteps.length > 0) {
        logger.debug('Adding execution steps from task metadata', {
          taskId: validatedTask.id,
          stepCount: executionSteps.length
        }, 'streamEventHandlers')
        useUIStateStore.getState().addSteps(executionSteps, event.context_id)
      }
    } catch (error) {
      if (error instanceof EventValidationError) {
        logger.error('Invalid task event', {
          eventType: error.eventType,
          message: error.message,
        }, 'streamEventHandlers')
        return
      }
      throw error
    }
  }

  if (hasArtifactsInData(event.data)) {
    const task = event.data.task as A2ATask | undefined

    if (!task) {
      logger.error('A2A protocol violation: artifacts present but task is missing', event.data, 'streamEventHandlers')
      return
    }

    if (!task.id || typeof task.id !== 'string') {
      logger.error('A2A protocol violation: Task.id is required', { task }, 'streamEventHandlers')
      return
    }

    let contextId: string
    try {
      contextId = validateContextId(event)
    } catch (error) {
      if (error instanceof EventValidationError) {
        logger.error(error.message, { event }, 'streamEventHandlers')
        return
      }
      throw error
    }

    const taskId: string = task.id

    logger.debug('Processing artifacts', { count: event.data.artifacts.length, taskId, contextId }, 'streamEventHandlers')

    const validatedArtifacts = validateArtifactsArray(event, logger)
    validatedArtifacts.forEach((validatedArtifact) => {
      useArtifactStore.getState().addArtifact(validatedArtifact, taskId, contextId)
      extractAndStoreSkill(validatedArtifact, contextId, taskId)
    })
  }
}

/**
 * Handle artifact created event
 */
export function handleArtifactCreatedEvent(event: BroadcastEvent): void {
  const currentContextId = useContextStore.getState().currentContextId

  logger.debug('Artifact created', { contextId: event.context_id, isCurrent: event.context_id === currentContextId }, 'streamEventHandlers')

  try {
    const validatedArtifact = validateArtifactEvent(event)
    const taskId = (event.data as Record<string, unknown>).task_id as string | undefined

    const uiState = useUIStateStore.getState()
    const toolName = validatedArtifact.metadata.tool_name as string | undefined

    let matchingExecution = uiState.findToolExecutionByArtifactId(validatedArtifact.artifactId)

    if (!matchingExecution && toolName) {
      const activeExecutions = uiState.getActiveToolExecutions()
      matchingExecution = activeExecutions.find(
        (e) => e.toolName === toolName
      )
    }

    if (!matchingExecution) {
      const activeExecutions = uiState.getActiveToolExecutions()
      matchingExecution = activeExecutions[0]
    }

    if (matchingExecution) {
      uiState.completeToolExecution(matchingExecution.id, validatedArtifact.artifactId)
      logger.debug('Completed execution', { id: matchingExecution.id, toolName }, 'streamEventHandlers')
    }

    useArtifactStore.getState().addArtifact(
      validatedArtifact,
      taskId,
      event.context_id
    )

    if (event.context_id && taskId) {
      extractAndStoreSkill(validatedArtifact, event.context_id, taskId)
    }
  } catch (error) {
    if (error instanceof EventValidationError) {
      logger.warn('Invalid artifact_created event', {
        eventType: error.eventType,
        message: error.message,
      }, 'streamEventHandlers')
      return
    }
    throw error
  }
}

/**
 * Handle skill loaded event
 */
export function handleSkillLoadedEvent(event: BroadcastEvent): void {
  logger.debug('Skill loaded', { contextId: event.context_id }, 'streamEventHandlers')

  if (!hasSkillInData(event.data)) {
    logger.warn('skill_loaded event missing skill fields', { eventData: event.data }, 'streamEventHandlers')
    return
  }

  try {
    const data = event.data
    const contextId = data.request_context?.execution?.context_id || event.context_id
    const taskId = data.task_id || data.request_context?.execution?.task_id

    if (contextId && taskId) {
      const skill = {
        id: data.skill_id,
        name: data.skill_name,
        description: data.description || '',
        tags: [],
      }

      import('@/stores/skill.store').then(({ useSkillStore }) => {
        useSkillStore.getState().addSkillToTask(contextId, taskId, skill)
        logger.debug('Skill added to store', { skillId: skill.id, taskId }, 'streamEventHandlers')
      })
    } else {
      logger.warn('Missing contextId or taskId in skill event', { contextId, taskId }, 'streamEventHandlers')
    }
  } catch (e) {
    logger.warn('Invalid skill in event', e, 'streamEventHandlers')
  }
}

/**
 * Handle message received event
 */
export function handleMessageReceivedEvent(event: BroadcastEvent): void {
  logger.debug('Message received', { contextId: event.context_id }, 'streamEventHandlers')

  if (event.context_id) {
    useContextStore.getState().updateMessageCount(event.context_id)
  }
}

/**
 * Handle task completed event with artifacts
 */
export function handleTaskCompletedEvent(event: BroadcastEvent): void {
  console.log('%c[TASK_COMPLETED] Raw event received', 'background: #ff9900; color: black; font-weight: bold;', {
    timestamp: new Date().toISOString(),
    contextId: event.context_id,
    rawData: event.data
  })

  logger.debug('Task completed', { contextId: event.context_id }, 'streamEventHandlers')

  if (!hasTaskInData(event.data)) {
    logger.error('A2A protocol violation: task_completed event missing task', event.data, 'streamEventHandlers')
    return
  }

  const task = event.data.task as A2ATask

  if (!task.id || typeof task.id !== 'string') {
    logger.error('A2A protocol violation: Task.id is required', { task }, 'streamEventHandlers')
    return
  }

  let contextId: string
  try {
    contextId = validateContextId(event)
  } catch (error) {
    if (error instanceof EventValidationError) {
      logger.error(error.message, { event }, 'streamEventHandlers')
      return
    }
    throw error
  }

  const taskId: string = task.id

  try {
    const validatedTask = validateTaskEvent(event)

    console.log('%c[TASK_COMPLETED] Validated task', 'background: #ff9900; color: black;', {
      taskId: validatedTask.id,
      status: validatedTask.status?.state,
      historyLength: validatedTask.history?.length || 0,
      historyRoles: validatedTask.history?.map(m => ({
        messageId: m.messageId,
        role: m.role,
        textPreview: m.parts?.[0]?.kind === 'text' ? (m.parts[0] as { text?: string }).text?.substring(0, 50) : 'N/A'
      })),
      hasMetadata: !!validatedTask.metadata,
      executionStepsCount: validatedTask.metadata?.executionSteps?.length || 0
    })

    useTaskStore.getState().updateTask(validatedTask)
    logger.debug('Updated task store from task_completed', { taskId }, 'streamEventHandlers')

    console.log('%c[TASK_COMPLETED] Clearing steps for task', 'background: #ff9900; color: black;', {
      taskId,
      stepsBeingCleared: useUIStateStore.getState().stepIdsByTask[taskId]?.length || 0
    })
    useUIStateStore.getState().clearStepsByTask(taskId)

    console.log('%c[TASK_COMPLETED] Clearing streaming state', 'background: #ff9900; color: black;', {
      currentStreamingTaskId: useUIStateStore.getState().activeStreamingTaskId,
      settingTo: null
    })

    useUIStateStore.getState().setStreaming(null)

    const steps = validatedTask.metadata?.executionSteps
    if (steps?.length) {
      useUIStateStore.getState().addSteps(steps, contextId)
    }
  } catch (error) {
    if (error instanceof EventValidationError) {
      logger.error('Invalid task in task_completed event', {
        eventType: error.eventType,
        message: error.message,
        taskId,
      }, 'streamEventHandlers')
    } else {
      throw error
    }
  }

  const validatedArtifacts = validateArtifactsArray(event, logger)

  logger.debug('Task completed with artifacts', {
    contextId,
    taskId,
    artifactCount: validatedArtifacts.length
  }, 'streamEventHandlers')

  if (validatedArtifacts.length > 0) {
    logger.debug('Processing task_completed artifacts', {
      artifactCount: validatedArtifacts.length,
      contextId,
      taskId
    }, 'streamEventHandlers')

    validatedArtifacts.forEach((validatedArtifact) => {
      useArtifactStore.getState().addArtifact(
        validatedArtifact,
        taskId,
        contextId
      )
      logger.debug('Added artifact from task_completed', {
        artifactId: validatedArtifact.artifactId,
        type: validatedArtifact.metadata.artifact_type
      }, 'streamEventHandlers')

      extractAndStoreSkill(validatedArtifact, contextId, taskId)
    })
  }

  if (validatedArtifacts.length > 0) {
    const toolName = task.metadata?.tool_name as string | undefined
    if (toolName) {
      const uiState = useUIStateStore.getState()
      const activeExecutions = uiState.getActiveToolExecutions()

      const matchingExecution = activeExecutions.find((e) => e.toolName === toolName)

      if (matchingExecution) {
        uiState.completeToolExecution(matchingExecution.id, validatedArtifacts[0].artifactId)
      }
    }
  }

  if (hasExecutionStepsInData(event.data)) {
    const steps = event.data.executionSteps as ExecutionStep[]
    if (steps.length > 0) {
      logger.debug('Adding execution steps from task_completed', {
        taskId,
        stepCount: steps.length
      }, 'streamEventHandlers')

      useUIStateStore.getState().addSteps(steps, contextId)
    }
  }

  fetchTaskWithExecutionSteps(taskId)
}

/**
 * Fetch task from API and merge execution steps into store
 * Called after task_completed to ensure authoritative state is available
 */
async function fetchTaskWithExecutionSteps(taskId: string): Promise<void> {
  try {
    const authHeader = useAuthStore.getState().getAuthHeader()
    await useTaskStore.getState().fetchTask(taskId, authHeader)

    const task = useTaskStore.getState().byId[taskId]
    if (task?.metadata?.executionSteps && task.metadata.executionSteps.length > 0) {
      logger.debug('Merging execution steps from API fetch', {
        taskId,
        stepCount: task.metadata.executionSteps.length
      }, 'streamEventHandlers')

      const contextId = task.contextId
      useUIStateStore.getState().addSteps(task.metadata.executionSteps, contextId)
    }
  } catch (error) {
    logger.warn('Failed to fetch task after completion', { taskId, error }, 'streamEventHandlers')
  }
}
