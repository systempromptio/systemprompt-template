/**
 * Type guard utilities for chat message streaming events.
 *
 * These type guards safely narrow discriminated union types
 * from the streaming API to specific event types.
 *
 * @module chat/helpers/typeGuards
 */

/**
 * Type guard for message events.
 *
 * @param event - The event to check
 * @returns True if event has role and parts properties
 */
export function isMessageEvent(event: unknown): event is Record<string, unknown> & { role: string; parts?: Array<{ kind: string }> } {
  return typeof event === 'object' && event !== null && 'role' in event
}

/**
 * Type guard for artifact update events.
 *
 * @param event - The event to check
 * @returns True if event is an artifact-update event
 */
export function isArtifactUpdateEvent(event: unknown): event is Record<string, unknown> & { kind: string; artifact?: unknown } {
  return typeof event === 'object' && event !== null && 'kind' in event && (event as Record<string, unknown>).kind === 'artifact-update'
}

/**
 * Type guard for task events.
 *
 * @param event - The event to check
 * @returns True if event is a task event
 */
export function isTaskEvent(event: unknown): event is Record<string, unknown> & { kind: string } {
  return typeof event === 'object' && event !== null && 'kind' in event && (event as Record<string, unknown>).kind === 'task'
}

/**
 * Type guard for status-update events.
 *
 * @param event - The event to check
 * @returns True if event is a status-update event with task state
 */
export function isStatusUpdateEvent(event: unknown): event is { kind: 'status-update'; taskId: string; contextId?: string; status: { state: string; message?: string } } {
  if (typeof event !== 'object' || event === null) return false
  const e = event as Record<string, unknown>
  return e.kind === 'status-update' && typeof e.taskId === 'string' && typeof e.status === 'object' && e.status !== null
}

/**
 * Extracts text content from message parts.
 *
 * @param parts - Array of message parts
 * @returns The text content or empty string if no text part found
 */
export function extractTextContent(parts: Array<{ kind: string; text?: string }>): string {
  const textPart = parts.find((p): p is { kind: 'text'; text: string } => p.kind === 'text' && typeof p.text === 'string')
  return textPart?.text ?? ''
}
