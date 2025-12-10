/**
 * SSE Event Validators
 *
 * Validates SSE events at the boundary before they enter the application.
 * Throws EventValidationError for malformed events, providing context for debugging.
 *
 * Architecture: SSE Event -> Validate -> Store -> Components
 * After validation, internal code trusts the types without defensive checks.
 */

import type { BroadcastEvent } from '@/types/sse'
import type { Task } from '@/types/task'
import type { Artifact } from '@/types/artifact'
import { toTask } from '@/types/task'
import { toArtifact } from '@/types/artifact'
import type { Task as A2ATask, Artifact as A2AArtifact } from '@a2a-js/sdk'

/**
 * Custom error for event validation failures.
 * Captures the event type, message, and raw event for debugging.
 */
export class EventValidationError extends Error {
  eventType: string
  rawEvent: BroadcastEvent

  constructor(eventType: string, message: string, rawEvent: BroadcastEvent) {
    super(`[${eventType}] ${message}`)
    this.name = 'EventValidationError'
    this.eventType = eventType
    this.rawEvent = rawEvent
  }
}

/**
 * Type guard: Check if event data has a task field
 */
export function hasTaskInData(
  data: unknown
): data is { task: A2ATask; [key: string]: unknown } {
  if (!data || typeof data !== 'object') return false
  return 'task' in data && data.task !== null && typeof data.task === 'object'
}

/**
 * Type guard: Check if event data has an artifact field
 */
export function hasArtifactInData(
  data: unknown
): data is { artifact: A2AArtifact; [key: string]: unknown } {
  if (!data || typeof data !== 'object') return false
  return 'artifact' in data && data.artifact !== null && typeof data.artifact === 'object'
}

/**
 * Type guard: Check if event data has an artifacts array field
 */
export function hasArtifactsInData(
  data: unknown
): data is { artifacts: A2AArtifact[]; [key: string]: unknown } {
  if (!data || typeof data !== 'object') return false
  return 'artifacts' in data && Array.isArray((data as { artifacts: unknown }).artifacts)
}

/**
 * Type guard: Check if event data has a message field
 */
export function hasMessageInData(
  data: unknown
): data is { message: unknown; [key: string]: unknown } {
  if (!data || typeof data !== 'object') return false
  return 'message' in data
}

/**
 * Type guard: Check if event data has execution steps
 */
export function hasExecutionStepsInData(
  data: unknown
): data is { executionSteps: unknown[]; [key: string]: unknown } {
  if (!data || typeof data !== 'object') return false
  return 'executionSteps' in data && Array.isArray((data as { executionSteps: unknown }).executionSteps)
}

/**
 * Type guard: Check if event data has skill fields
 */
export function hasSkillInData(
  data: unknown
): data is { skill_id: string; skill_name: string; [key: string]: unknown } {
  if (!data || typeof data !== 'object') return false
  const d = data as Record<string, unknown>
  return (
    'skill_id' in d &&
    typeof d.skill_id === 'string' &&
    'skill_name' in d &&
    typeof d.skill_name === 'string'
  )
}

/**
 * Validate and extract task from SSE event.
 * Throws EventValidationError if event is malformed.
 */
export function validateTaskEvent(event: BroadcastEvent): Task {
  if (!hasTaskInData(event.data)) {
    throw new EventValidationError(
      event.event_type,
      `Event data missing task field. Received keys: ${Object.keys(event.data || {}).join(', ')}`,
      event
    )
  }

  try {
    return toTask(event.data.task)
  } catch (error) {
    throw new EventValidationError(
      event.event_type,
      `Invalid task structure: ${error instanceof Error ? error.message : String(error)}`,
      event
    )
  }
}

/**
 * Validate and extract artifact from SSE event.
 * Throws EventValidationError if event is malformed.
 */
export function validateArtifactEvent(event: BroadcastEvent): Artifact {
  if (!hasArtifactInData(event.data)) {
    throw new EventValidationError(
      event.event_type,
      `Event data missing artifact field. Received keys: ${Object.keys(event.data || {}).join(', ')}`,
      event
    )
  }

  try {
    return toArtifact(event.data.artifact)
  } catch (error) {
    throw new EventValidationError(
      event.event_type,
      `Invalid artifact structure: ${error instanceof Error ? error.message : String(error)}`,
      event
    )
  }
}

/**
 * Validate and extract artifacts array from SSE event.
 * Returns empty array if no artifacts present.
 * Logs warnings for individual invalid artifacts but doesn't throw.
 */
export function validateArtifactsArray(
  event: BroadcastEvent,
  logger?: { warn: (msg: string, data: unknown, source: string) => void }
): Artifact[] {
  if (!hasArtifactsInData(event.data)) {
    return []
  }

  const validated: Artifact[] = []
  event.data.artifacts.forEach((rawArtifact, index) => {
    try {
      validated.push(toArtifact(rawArtifact))
    } catch (e) {
      logger?.warn('Invalid artifact in array', { index, error: e }, 'eventValidators')
    }
  })

  return validated
}

/**
 * Validate required context_id in event.
 * Throws EventValidationError if missing or invalid.
 */
export function validateContextId(event: BroadcastEvent): string {
  if (!event.context_id || typeof event.context_id !== 'string') {
    throw new EventValidationError(
      event.event_type,
      'A2A protocol violation: context_id is required',
      event
    )
  }
  return event.context_id
}

/**
 * Validate required task_id in event data.
 * Throws EventValidationError if missing or invalid.
 */
export function validateTaskIdInData(event: BroadcastEvent): string {
  const taskId = (event.data as Record<string, unknown>)?.task_id
  if (!taskId || typeof taskId !== 'string') {
    throw new EventValidationError(
      event.event_type,
      'Event data missing task_id field',
      event
    )
  }
  return taskId
}

/**
 * Combined validation for task events with context.
 * Returns validated task, taskId, and contextId.
 */
export function validateTaskEventWithContext(event: BroadcastEvent): {
  task: Task
  taskId: string
  contextId: string
} {
  const task = validateTaskEvent(event)
  const contextId = validateContextId(event)

  return {
    task,
    taskId: task.id,
    contextId,
  }
}

/**
 * Combined validation for artifact events with context.
 * Returns validated artifact, taskId (optional), and contextId.
 */
export function validateArtifactEventWithContext(event: BroadcastEvent): {
  artifact: Artifact
  taskId: string | undefined
  contextId: string
} {
  const artifact = validateArtifactEvent(event)
  const contextId = validateContextId(event)
  const taskId = (event.data as Record<string, unknown>)?.task_id as string | undefined

  return {
    artifact,
    taskId,
    contextId,
  }
}
