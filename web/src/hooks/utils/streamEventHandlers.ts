/**
 * Stream Event Handler Utilities
 *
 * Extracted handlers for different SSE event types.
 * Each handler is responsible for a single event type.
 */

import { useTaskStore } from '@/stores/task.store'
import { useArtifactStore } from '@/stores/artifact.store'
import { useToolExecutionStore } from '@/stores/toolExecution.store'
import { useContextStore } from '@/stores/context.store'
import { toTask } from '@/types/task'
import { toArtifact } from '@/types/artifact'
import type { Task as A2ATask, Artifact as A2AArtifact } from '@a2a-js/sdk'
import type { BroadcastEvent } from '@/types/sse'
import { logger } from '@/lib/logger'
import { extractAndStoreSkill } from '@/lib/utils/extractArtifactSkills'

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
      contexts.forEach((ctx: any) => {
        if (ctx.context_id && ctx.message_count !== undefined) {
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
 * Handle task event - updates task store
 */
export function handleTaskEvent(event: BroadcastEvent): void {
  if ('task' in event.data) {
    try {
      const rawTask = event.data.task as A2ATask
      const validatedTask = toTask(rawTask)
      logger.debug('Updating task store', { taskId: validatedTask.id }, 'streamEventHandlers')
      useTaskStore.getState().updateTask(validatedTask)
    } catch (e) {
      logger.warn('Invalid task in event', e, 'streamEventHandlers')
    }
  }

  if ('artifacts' in event.data && Array.isArray(event.data.artifacts)) {
    const task = event.data.task as A2ATask | undefined

    if (!task) {
      logger.error('A2A protocol violation: artifacts present but task is missing', event.data, 'streamEventHandlers')
      throw new Error('A2A protocol violation: artifacts array present but task is missing')
    }

    if (!task.id || typeof task.id !== 'string') {
      logger.error('A2A protocol violation: Task.id is required', { task }, 'streamEventHandlers')
      throw new Error('A2A protocol violation: Task.id is required but missing or invalid')
    }

    if (!event.context_id || typeof event.context_id !== 'string') {
      logger.error('A2A protocol violation: context_id is required', { event }, 'streamEventHandlers')
      throw new Error('A2A protocol violation: context_id is required but missing or invalid')
    }

    try {
      const artifacts = event.data.artifacts as A2AArtifact[]
      const taskId: string = task.id
      const contextId: string = event.context_id

      logger.debug('Processing artifacts', { count: artifacts.length, taskId, contextId }, 'streamEventHandlers')

      artifacts.forEach((rawArtifact) => {
        try {
          const validatedArtifact = toArtifact(rawArtifact)
          useArtifactStore.getState().addArtifact(validatedArtifact, taskId, contextId)
          extractAndStoreSkill(validatedArtifact, contextId, taskId)
        } catch (e) {
          logger.warn('Invalid artifact in event', e, 'streamEventHandlers')
        }
      })
    } catch (e) {
      logger.error('Failed to process artifacts', e, 'streamEventHandlers')
      throw e
    }
  }
}

/**
 * Handle artifact created event
 */
export function handleArtifactCreatedEvent(event: BroadcastEvent): void {
  const currentContextId = useContextStore.getState().currentContextId

  logger.debug('Artifact created', { contextId: event.context_id, isCurrent: event.context_id === currentContextId }, 'streamEventHandlers')

  if ('artifact' in event.data) {
    try {
      const rawArtifact = event.data.artifact as A2AArtifact
      const validatedArtifact = toArtifact(rawArtifact)
      const taskId = event.data.task_id as string | undefined
      const renderBehavior = validatedArtifact.metadata.render_behavior || 'both'

      const executions = useToolExecutionStore.getState().getQueue()
      const toolName = validatedArtifact.metadata.tool_name as string | undefined

      let matchingExecution = useToolExecutionStore.getState().findByArtifactId(validatedArtifact.artifactId)

      if (!matchingExecution && toolName) {
        matchingExecution = executions.find(
          (e) => (e.status === 'pending' || e.status === 'executing') && e.toolName === toolName
        )
      }

      if (!matchingExecution) {
        matchingExecution = executions.find(
          (e) => e.status === 'pending' || e.status === 'executing'
        )
      }

      if (matchingExecution) {
        useToolExecutionStore.getState().completeExecution(matchingExecution.id, validatedArtifact)
        logger.debug('Completed execution', { id: matchingExecution.id, toolName }, 'streamEventHandlers')
      }

      if (renderBehavior === 'inline' || renderBehavior === 'both') {
        useArtifactStore.getState().addArtifact(
          validatedArtifact,
          taskId,
          event.context_id
        )
      }

      // Extract and store skill metadata from artifact
      if (event.context_id && taskId) {
        extractAndStoreSkill(validatedArtifact, event.context_id, taskId)
      }
    } catch (e) {
      logger.warn('Invalid artifact in event', e, 'streamEventHandlers')
    }
  }
}

/**
 * Handle skill loaded event
 */
export function handleSkillLoadedEvent(event: BroadcastEvent): void {
  console.log('[DEBUG] handleSkillLoadedEvent - Processing skill event:', {
    event,
    contextId: event.context_id,
    hasSkillId: 'skill_id' in event.data,
    hasSkillName: 'skill_name' in event.data,
    eventData: event.data
  })

  logger.debug('Skill loaded', { contextId: event.context_id }, 'streamEventHandlers')

  if ('skill_id' in event.data && 'skill_name' in event.data) {
    try {
      const data = event.data as any
      const contextId = data.request_context?.execution?.context_id || event.context_id
      const taskId = data.task_id || data.request_context?.execution?.task_id

      console.log('[DEBUG] handleSkillLoadedEvent - Extracted skill data:', {
        contextId,
        taskId,
        skillId: data.skill_id,
        skillName: data.skill_name,
        hasContextId: !!contextId,
        hasTaskId: !!taskId,
        requestContext: data.request_context
      })

      if (contextId && taskId) {
        const skill = {
          id: data.skill_id as string,
          name: data.skill_name as string,
          description: data.description as string || '',
          tags: [],
        }

        console.log('[DEBUG] handleSkillLoadedEvent - Adding skill to store:', {
          contextId,
          taskId,
          skill
        })

        import('@/stores/skill.store').then(({ useSkillStore }) => {
          useSkillStore.getState().addSkillToTask(contextId, taskId, skill)
          console.log('[DEBUG] handleSkillLoadedEvent - Skill added to store. Current state:', {
            byContext: useSkillStore.getState().byContext,
            byId: useSkillStore.getState().byId
          })
          logger.debug('Skill added to store', { skillId: skill.id, taskId }, 'streamEventHandlers')
        })
      } else {
        console.warn('[DEBUG] handleSkillLoadedEvent - Missing contextId or taskId:', {
          contextId,
          taskId,
          rawData: data,
          event
        })
        logger.warn('Missing contextId or taskId in skill event', { contextId, taskId }, 'streamEventHandlers')
      }
    } catch (e) {
      console.error('[DEBUG] handleSkillLoadedEvent - Error:', e)
      logger.warn('Invalid skill in event', e, 'streamEventHandlers')
    }
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
  logger.debug('Task completed', { contextId: event.context_id }, 'streamEventHandlers')

  if (!('task' in event.data) || !('artifacts' in event.data) || !Array.isArray(event.data.artifacts)) {
    return
  }

  const task = event.data.task as A2ATask | undefined
  const artifacts = event.data.artifacts as A2AArtifact[]

  if (!task) {
    logger.error('A2A protocol violation: task_completed event missing task', event.data, 'streamEventHandlers')
    throw new Error('A2A protocol violation: task_completed event requires task')
  }

  if (!task.id || typeof task.id !== 'string') {
    logger.error('A2A protocol violation: Task.id is required', { task }, 'streamEventHandlers')
    throw new Error('A2A protocol violation: Task.id is required but missing or invalid')
  }

  if (!event.context_id || typeof event.context_id !== 'string') {
    logger.error('A2A protocol violation: context_id is required', { event }, 'streamEventHandlers')
    throw new Error('A2A protocol violation: context_id is required but missing or invalid')
  }

  const taskId: string = task.id
  const contextId: string = event.context_id

  logger.debug('Task completed with artifacts', {
    contextId,
    taskId,
    artifactCount: artifacts.length
  }, 'streamEventHandlers')

  // Extract skill metadata from artifacts
  if (artifacts.length > 0) {
    console.log('[TASK COMPLETED DEBUG] Processing artifacts for skill extraction:', {
      artifactCount: artifacts.length,
      contextId,
      taskId
    })

    artifacts.forEach((rawArtifact, index) => {
      console.log(`[TASK COMPLETED DEBUG] Processing artifact ${index}:`, rawArtifact)
      try {
        const validatedArtifact = toArtifact(rawArtifact)
        console.log(`[TASK COMPLETED DEBUG] Validated artifact ${index}:`, validatedArtifact)
        extractAndStoreSkill(validatedArtifact, contextId, taskId)
      } catch (e) {
        console.error(`[TASK COMPLETED DEBUG] Failed to extract skill from artifact ${index}:`, e)
        logger.warn('Failed to extract skill from artifact', e, 'streamEventHandlers')
      }
    })
  }

  if (artifacts.length > 0) {
    const toolName = task.metadata?.tool_name as string | undefined
    if (toolName) {
      const { completeExecution, failExecution, executions } = useToolExecutionStore.getState()

      const matchingExecution = executions.find((e) => {
        if (e.toolName !== toolName) return false
        if (e.status === 'completed' || e.status === 'error') return false
        return true
      })

      if (matchingExecution) {
        try {
          const artifact = toArtifact(artifacts[0])
          completeExecution(matchingExecution.id, artifact)
        } catch (err) {
          logger.error('Failed to complete execution', err, 'streamEventHandlers')
          const errorMessage = err instanceof Error ? err.message : 'Unknown error'
          failExecution(matchingExecution.id, errorMessage)
        }
      }
    }
  }
}
