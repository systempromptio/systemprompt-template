/**
 * Streaming event utilities and helpers.
 *
 * Provides utilities for processing, filtering, and categorizing
 * events from streaming APIs and WebSocket connections.
 *
 * @module utils/stream-events
 */

import type { Task, Message, TaskStatusUpdateEvent, TaskArtifactUpdateEvent } from '@a2a-js/sdk'

/**
 * Union type of all possible streaming events.
 */
export type StreamEvent = Message | Task | TaskStatusUpdateEvent | TaskArtifactUpdateEvent

/**
 * Check if an event is a message from an agent.
 *
 * @param event - The event to check
 * @returns True if event is a Message from agent role
 *
 * @example
 * ```typescript
 * for await (const event of stream) {
 *   if (isAgentMessage(event)) {
 *     console.log(event.content)
 *   }
 * }
 * ```
 */
export function isAgentMessage(event: unknown): event is Message {
  if (!event || typeof event !== 'object') return false

  const obj = event as Record<string, unknown>
  return obj.type === 'message' && obj.role === 'agent'
}

/**
 * Check if an event is a message from the user.
 *
 * @param event - The event to check
 * @returns True if event is a Message from user role
 *
 * @example
 * ```typescript
 * if (isUserMessage(event)) {
 *   // Handle user message
 * }
 * ```
 */
export function isUserMessage(event: unknown): event is Message {
  if (!event || typeof event !== 'object') return false

  const obj = event as Record<string, unknown>
  return obj.type === 'message' && obj.role === 'user'
}

/**
 * Check if an event is a task update.
 *
 * @param event - The event to check
 * @returns True if event contains task information
 *
 * @example
 * ```typescript
 * if (isTaskUpdate(event)) {
 *   updateTaskState(event.task)
 * }
 * ```
 */
export function isTaskUpdate(event: unknown): event is Task {
  if (!event || typeof event !== 'object') return false

  const obj = event as Record<string, unknown>
  return (
    typeof obj.id === 'string' &&
    obj.status !== null &&
    typeof obj.status === 'object'
  )
}

/**
 * Check if an event is a task status change.
 *
 * @param event - The event to check
 * @returns True if event is a TaskStatusUpdateEvent
 *
 * @example
 * ```typescript
 * if (isTaskStatusUpdate(event)) {
 *   logTaskStateChange(event)
 * }
 * ```
 */
export function isTaskStatusUpdate(event: unknown): event is TaskStatusUpdateEvent {
  if (!event || typeof event !== 'object') return false

  const obj = event as Record<string, unknown>
  return (
    obj.type === 'task_status_update' &&
    typeof obj.task_id === 'string' &&
    obj.status !== null
  )
}

/**
 * Check if an event is a task artifact update.
 *
 * @param event - The event to check
 * @returns True if event is a TaskArtifactUpdateEvent
 *
 * @example
 * ```typescript
 * if (isTaskArtifactUpdate(event)) {
 *   displayArtifact(event.artifact)
 * }
 * ```
 */
export function isTaskArtifactUpdate(event: unknown): event is TaskArtifactUpdateEvent {
  if (!event || typeof event !== 'object') return false

  const obj = event as Record<string, unknown>
  return (
    obj.type === 'task_artifact_update' &&
    typeof obj.task_id === 'string' &&
    obj.artifact !== null
  )
}

/**
 * Filter stream events by type.
 *
 * Returns new array containing only events matching the type predicate.
 *
 * @template T - The expected event type
 * @param events - Array of events to filter
 * @param predicate - Type guard function
 * @returns Filtered array of events
 *
 * @example
 * ```typescript
 * const messages = filterStreamEvents(events, isAgentMessage)
 * const taskUpdates = filterStreamEvents(events, isTaskUpdate)
 * ```
 */
export function filterStreamEvents<T>(
  events: unknown[],
  predicate: (event: unknown) => event is T
): T[] {
  return events.filter(predicate)
}

/**
 * Get all messages from a stream of events.
 *
 * @param events - Array of mixed events
 * @returns Array of Message events
 *
 * @example
 * ```typescript
 * const allMessages = extractMessages(streamEvents)
 * const agentMessages = allMessages.filter(m => m.role === 'agent')
 * ```
 */
export function extractMessages(events: unknown[]): Message[] {
  return filterStreamEvents(events, (e): e is Message => {
    if (!e || typeof e !== 'object') return false
    const obj = e as Record<string, unknown>
    return obj.type === 'message' && (obj.role === 'agent' || obj.role === 'user')
  })
}

/**
 * Get all task updates from a stream of events.
 *
 * @param events - Array of mixed events
 * @returns Array of Task events
 *
 * @example
 * ```typescript
 * const taskUpdates = extractTasks(streamEvents)
 * const failedTasks = taskUpdates.filter(t => t.status.state === 'failed')
 * ```
 */
export function extractTasks(events: unknown[]): Task[] {
  return filterStreamEvents(events, isTaskUpdate)
}

/**
 * Get all status updates from a stream of events.
 *
 * @param events - Array of mixed events
 * @returns Array of TaskStatusUpdateEvent events
 *
 * @example
 * ```typescript
 * const statusUpdates = extractStatusUpdates(streamEvents)
 * ```
 */
export function extractStatusUpdates(events: unknown[]): TaskStatusUpdateEvent[] {
  return filterStreamEvents(events, isTaskStatusUpdate)
}

/**
 * Get all artifact updates from a stream of events.
 *
 * @param events - Array of mixed events
 * @returns Array of TaskArtifactUpdateEvent events
 *
 * @example
 * ```typescript
 * const artifacts = extractArtifacts(streamEvents)
 * ```
 */
export function extractArtifacts(events: unknown[]): TaskArtifactUpdateEvent[] {
  return filterStreamEvents(events, isTaskArtifactUpdate)
}

/**
 * Group stream events by type.
 *
 * Returns object with separated event arrays by category.
 *
 * @param events - Array of mixed events
 * @returns Object with arrays of events grouped by type
 *
 * @example
 * ```typescript
 * const grouped = groupStreamEvents(events)
 * console.log(grouped.messages.length) // Message count
 * console.log(grouped.tasks.length) // Task count
 * ```
 */
export function groupStreamEvents(events: unknown[]) {
  return {
    messages: extractMessages(events),
    tasks: extractTasks(events),
    statusUpdates: extractStatusUpdates(events),
    artifacts: extractArtifacts(events),
  }
}

/**
 * Find the last message of a specific role in events.
 *
 * Useful for finding the most recent agent response or user input.
 *
 * @param events - Array of events
 * @param role - Message role to find ('agent', 'user')
 * @returns Last message with given role, or undefined
 *
 * @example
 * ```typescript
 * const lastResponse = findLastMessage(events, 'agent')
 * if (lastResponse) {
 *   console.log('Agent said:', lastResponse.content)
 * }
 * ```
 */
export function findLastMessage(events: unknown[], role: string): Message | undefined {
  const messages = extractMessages(events)
  return [...messages].reverse().find(m => m.role === role)
}

/**
 * Find the last task update in events.
 *
 * Returns the most recent task state.
 *
 * @param events - Array of events
 * @returns Last Task event, or undefined
 *
 * @example
 * ```typescript
 * const lastTask = findLastTask(events)
 * if (lastTask?.status.state === 'completed') {
 *   showCompletion()
 * }
 * ```
 */
export function findLastTask(events: unknown[]): Task | undefined {
  const tasks = extractTasks(events)
  return tasks[tasks.length - 1]
}

/**
 * Find a task by ID in events.
 *
 * @param events - Array of events
 * @param taskId - ID to search for
 * @returns Task with matching ID, or undefined
 *
 * @example
 * ```typescript
 * const task = findTaskById(events, '123')
 * ```
 */
export function findTaskById(events: unknown[], taskId: string): Task | undefined {
  return extractTasks(events).find(t => t.id === taskId)
}

/**
 * Get all agents mentioned in stream events.
 *
 * Extracts unique agent names from messages and status metadata.
 *
 * @param events - Array of events
 * @returns Set of unique agent identifiers
 *
 * @example
 * ```typescript
 * const agents = getAgentsFromEvents(events)
 * console.log(`Involved agents: ${[...agents].join(', ')}`)
 * ```
 */
export function getAgentsFromEvents(events: unknown[]): Set<string> {
  const agents = new Set<string>()

  const messages = extractMessages(events)
  messages.forEach(msg => {
    if (msg.role === 'agent') {
      const metadata = msg.metadata as Record<string, unknown> | undefined
      if (metadata?.agent_id) {
        agents.add(String(metadata.agent_id))
      }
    }
  })

  const tasks = extractTasks(events)
  tasks.forEach(task => {
    const taskMetadata = task.metadata as Record<string, unknown> | undefined
    if (taskMetadata?.agent_id) {
      agents.add(String(taskMetadata.agent_id))
    }
  })

  return agents
}

/**
 * Calculate total message content length from events.
 *
 * Sums up all message text lengths for bandwidth/memory assessment.
 *
 * @param events - Array of events
 * @returns Total character count across all messages
 *
 * @example
 * ```typescript
 * const totalLen = getTotalMessageLength(events)
 * console.log(`Streamed ${totalLen} characters`)
 * ```
 */
export function getTotalMessageLength(events: unknown[]): number {
  return extractMessages(events).reduce((sum, msg) => {
    const textLength = msg.parts?.reduce((total, part) => {
      if (part.kind === 'text') {
        const textPart = part as unknown as Record<string, unknown>
        if (typeof textPart.text === 'string') {
          return total + textPart.text.length
        }
      }
      return total
    }, 0) || 0
    return sum + textLength
  }, 0)
}

/**
 * Check if stream contains any errors.
 *
 * Looks for failed tasks or error indicators in events.
 *
 * @param events - Array of events
 * @returns True if any errors found
 *
 * @example
 * ```typescript
 * if (hasStreamErrors(events)) {
 *   showErrorMessage()
 * }
 * ```
 */
export function hasStreamErrors(events: unknown[]): boolean {
  const tasks = extractTasks(events)
  return tasks.some(t => ['failed', 'rejected'].includes(t.status?.state || ''))
}

/**
 * Filter stream events by timestamp range.
 *
 * @param events - Array of events
 * @param startMs - Start time in milliseconds
 * @param endMs - End time in milliseconds
 * @returns Events within the time range
 *
 * @example
 * ```typescript
 * const recent = filterByTimeRange(events, Date.now() - 60000, Date.now())
 * ```
 */
export function filterByTimeRange(
  events: unknown[],
  startMs: number,
  endMs: number
): unknown[] {
  return events.filter(e => {
    if (!e || typeof e !== 'object') return false

    const obj = e as Record<string, unknown>
    const timestamp = obj.timestamp as number | undefined

    if (!timestamp) return false

    return timestamp >= startMs && timestamp <= endMs
  })
}
