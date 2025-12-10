import type { Task as A2ATask, Artifact as A2AArtifact, Message } from '@a2a-js/sdk'

export interface BroadcastEvent {
  event_type: string
  context_id: string
  user_id: string
  data: BroadcastEventData
  timestamp: string
}

export type BroadcastEventData =
  | TaskEventData
  | ArtifactEventData
  | MessageEventData
  | SkillEventData
  | ExecutionStepEventData
  | Record<string, unknown>

export interface TaskEventData {
  task: A2ATask
  artifacts?: A2AArtifact[]
  context_id?: string
}

export interface ArtifactEventData {
  artifact: A2AArtifact
  task_id?: string
  context_id?: string
}

export interface MessageEventData {
  message?: Message
  context_id?: string
}

export interface SkillEventData {
  skill_id: string
  skill_name: string
  description?: string
  task_id?: string
  request_context?: {
    execution?: {
      context_id?: string
      task_id?: string
    }
  }
}

export interface ExecutionStepEventData {
  step: {
    stepId: string
    status: string
    title?: string
    taskId?: string
  }
  context_id?: string
}

export interface CurrentAgentEvent {
  type: 'current_agent'
  context_id: string
  /** Agent name, or null to clear/remove agent assignment */
  agent_name: string | null
  timestamp: string
}

/**
 * Type guard to check if event data contains a task
 */
export function hasTaskInData(data: BroadcastEventData): data is TaskEventData {
  return (
    typeof data === 'object' &&
    data !== null &&
    'task' in data &&
    typeof data.task === 'object' &&
    data.task !== null
  )
}

/**
 * Type guard to check if event data contains an artifact
 */
export function hasArtifactInData(data: BroadcastEventData): data is ArtifactEventData {
  return (
    typeof data === 'object' &&
    data !== null &&
    'artifact' in data &&
    typeof data.artifact === 'object' &&
    data.artifact !== null
  )
}

/**
 * Type guard to check if event data contains skill info
 */
export function hasSkillInData(data: BroadcastEventData): data is SkillEventData {
  return (
    typeof data === 'object' &&
    data !== null &&
    'skill_id' in data &&
    typeof data.skill_id === 'string' &&
    'skill_name' in data &&
    typeof data.skill_name === 'string'
  )
}

/**
 * Type guard to check if event data contains an execution step
 */
export function hasStepInData(data: BroadcastEventData): data is ExecutionStepEventData {
  return (
    typeof data === 'object' &&
    data !== null &&
    'step' in data &&
    typeof data.step === 'object' &&
    data.step !== null
  )
}

/**
 * Type guard to check if an event is a CurrentAgentEvent
 */
export function isCurrentAgentEvent(event: unknown): event is CurrentAgentEvent {
  return (
    typeof event === 'object' &&
    event !== null &&
    'type' in event &&
    event.type === 'current_agent' &&
    'context_id' in event &&
    typeof event.context_id === 'string' &&
    'agent_name' in event &&
    (typeof event.agent_name === 'string' || event.agent_name === null) &&
    'timestamp' in event &&
    typeof event.timestamp === 'string'
  )
}
