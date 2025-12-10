/**
 * SSE Event Types - Server-sent events for real-time context updates
 */
export const EventType = {
  TASK_COMPLETED: 'task_completed',
  TASK_CREATED: 'task_created',
  TASK_STATUS_CHANGED: 'task_status_changed',
  CONTEXT_CREATED: 'context_created',
  CONTEXT_UPDATED: 'context_updated',
  CONTEXT_DELETED: 'context_deleted',
  CURRENT_AGENT: 'current_agent',
  HEARTBEAT: 'heartbeat',
  MESSAGE_ADDED: 'message_added',
  ARTIFACT_CREATED: 'artifact_created',
  TOOL_EXECUTION_COMPLETED: 'tool_execution_completed',
  SKILL_LOADED: 'skill_loaded',
  EXECUTION_STEP: 'execution_step',
} as const

export type EventType = typeof EventType[keyof typeof EventType]

/**
 * Tool Execution Status - States during MCP tool execution lifecycle
 */
export const ExecutionStatus = {
  PENDING: 'pending',
  EXECUTING: 'executing',
  COMPLETED: 'completed',
  ERROR: 'error',
} as const

export type ExecutionStatus = typeof ExecutionStatus[keyof typeof ExecutionStatus]
