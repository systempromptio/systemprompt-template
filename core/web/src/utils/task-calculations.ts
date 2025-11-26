/**
 * Task duration and calculation utilities.
 *
 * Centralizes task duration calculations, status formatting, and time-based
 * operations to eliminate duplication across components and stores.
 *
 * @module utils/task-calculations
 */

import type { Task } from '@/types/task'

/**
 * Parses SQLite and RFC3339 datetime strings to milliseconds.
 *
 * Handles both SQLite format (YYYY-MM-DD HH:MM:SS.SSS) and
 * RFC3339 format (2024-01-01T12:00:00Z). Returns 0 for invalid input.
 *
 * @param dateStr - DateTime string to parse
 * @returns Milliseconds since epoch, or 0 if parsing fails
 *
 * @example
 * ```typescript
 * const ms = parseSQLiteDateTime('2024-01-01 12:00:00.000')
 * const ms2 = parseSQLiteDateTime('2024-01-01T12:00:00Z')
 * ```
 */
export function parseSQLiteDateTime(dateStr: string | null): number {
  if (!dateStr) return 0

  try {
    const sqliteFormat = /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}/
    if (sqliteFormat.test(dateStr)) {
      return new Date(dateStr.replace(' ', 'T')).getTime()
    }
    return new Date(dateStr).getTime()
  } catch {
    return 0
  }
}

/**
 * Formats a duration in milliseconds as a human-readable string.
 *
 * Converts milliseconds to the most appropriate unit (hours, minutes, seconds, ms).
 * Examples: "1h 30m", "45m 30s", "5s", "234ms"
 *
 * @param ms - Duration in milliseconds
 * @returns Formatted duration string
 *
 * @example
 * ```typescript
 * formatDuration(5400000) // "1h 30m"
 * formatDuration(2700) // "2s 700ms"
 * formatDuration(45000) // "45s"
 * ```
 */
export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`

  const totalSeconds = Math.floor(ms / 1000)
  const hours = Math.floor(totalSeconds / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = totalSeconds % 60

  if (hours > 0) {
    return minutes > 0 ? `${hours}h ${minutes}m` : `${hours}h`
  }
  if (minutes > 0) {
    return seconds > 0 ? `${minutes}m ${seconds}s` : `${minutes}m`
  }
  return `${seconds}s`
}

/**
 * Calculates task duration with multiple fallback strategies.
 *
 * Attempts to determine task duration using:
 * 1. `execution_time_ms` from metadata (most reliable)
 * 2. `started_at` and `completed_at` timestamps
 * 3. `created_at` and status timestamp
 * 4. Returns "In Progress" if task is still running
 *
 * @param task - Task object with status and timestamps
 * @returns Formatted duration string or "In Progress" if incomplete
 *
 * @example
 * ```typescript
 * const task = {
 *   created_at: '2024-01-01T10:00:00Z',
 *   completed_at: '2024-01-01T11:30:00Z'
 * }
 * calculateTaskDuration(task) // "1h 30m"
 * ```
 */
export function calculateTaskDuration(task: Task): string {
  if (!task.status) return 'Unknown'

  const isCompleted = ['completed', 'failed', 'rejected'].includes(task.status.state)

  if (!isCompleted) {
    return 'In Progress'
  }

  const metadata = task.metadata as Record<string, unknown> | undefined
  if (metadata?.execution_time_ms && typeof metadata.execution_time_ms === 'number') {
    return formatDuration(metadata.execution_time_ms)
  }

  const startedAt = metadata?.started_at as string | undefined
  const completedAt = metadata?.completed_at as string | undefined
  const startedMs = startedAt ? parseSQLiteDateTime(startedAt) : 0
  const completedMs = completedAt ? parseSQLiteDateTime(completedAt) : 0

  if (startedMs > 0 && completedMs > 0) {
    return formatDuration(completedMs - startedMs)
  }

  return 'Unknown'
}

/**
 * Task status display information.
 */
export interface TaskStatusInfo {
  /**
   * Human-readable status label
   */
  label: string

  /**
   * CSS class for styling
   */
  className: string

  /**
   * Color variant for UI (success, danger, warning, info)
   */
  color: 'success' | 'danger' | 'warning' | 'info' | 'default'

  /**
   * Icon name (e.g., 'checkmark', 'error', 'clock')
   */
  icon: string
}

/**
 * Map of task states to display information.
 *
 * @constant
 */
const STATUS_INFO_MAP: Record<string, TaskStatusInfo> = {
  submitted: {
    label: 'Submitted',
    className: 'status-submitted',
    color: 'info',
    icon: 'send',
  },
  working: {
    label: 'Working',
    className: 'status-working',
    color: 'info',
    icon: 'hourglass',
  },
  'input-required': {
    label: 'Input Required',
    className: 'status-input-required',
    color: 'warning',
    icon: 'question',
  },
  completed: {
    label: 'Completed',
    className: 'status-completed',
    color: 'success',
    icon: 'checkmark',
  },
  failed: {
    label: 'Failed',
    className: 'status-failed',
    color: 'danger',
    icon: 'error',
  },
  rejected: {
    label: 'Rejected',
    className: 'status-rejected',
    color: 'danger',
    icon: 'block',
  },
  canceled: {
    label: 'Canceled',
    className: 'status-canceled',
    color: 'default',
    icon: 'close',
  },
  'auth-required': {
    label: 'Auth Required',
    className: 'status-auth-required',
    color: 'warning',
    icon: 'lock',
  },
}

/**
 * Get formatted status information for a task state.
 *
 * Returns display label, styling, and icon information for the given task state.
 * Unknown states return a default 'Unknown' status.
 *
 * @param state - Task status state string
 * @returns Status display information
 *
 * @example
 * ```typescript
 * const info = getTaskStatusInfo('completed')
 * console.log(info.label) // "Completed"
 * console.log(info.color) // "success"
 * console.log(info.icon) // "checkmark"
 * ```
 */
export function getTaskStatusInfo(state: string): TaskStatusInfo {
  return STATUS_INFO_MAP[state] || {
    label: 'Unknown',
    className: 'status-unknown',
    color: 'default',
    icon: 'help',
  }
}

/**
 * Check if a task is in a completed state.
 *
 * Returns true for any final state (completed, failed, rejected, canceled).
 *
 * @param state - Task status state
 * @returns True if task is in a terminal state
 *
 * @example
 * ```typescript
 * isTaskCompleted('completed') // true
 * isTaskCompleted('failed') // true
 * isTaskCompleted('working') // false
 * ```
 */
export function isTaskCompleted(state: string): boolean {
  return ['completed', 'failed', 'rejected', 'canceled'].includes(state)
}

/**
 * Check if a task is currently running.
 *
 * Returns true for in-progress states (working, submitted, input-required).
 *
 * @param state - Task status state
 * @returns True if task is actively running
 *
 * @example
 * ```typescript
 * isTaskRunning('working') // true
 * isTaskRunning('submitted') // true
 * isTaskRunning('completed') // false
 * ```
 */
export function isTaskRunning(state: string): boolean {
  return ['working', 'submitted', 'input-required'].includes(state)
}

/**
 * Format timestamp for display with consistent formatting.
 *
 * Returns locale-appropriate date/time string or empty string for null input.
 * Optionally includes time component.
 *
 * @param timestamp - ISO timestamp string or null
 * @param includeTime - Whether to include time (default: false)
 * @returns Formatted date string
 *
 * @example
 * ```typescript
 * formatTaskTimestamp('2024-01-01T10:30:00Z') // "1/1/2024"
 * formatTaskTimestamp('2024-01-01T10:30:00Z', true) // "1/1/2024, 10:30:00 AM"
 * ```
 */
export function formatTaskTimestamp(timestamp: string | null, includeTime = false): string {
  if (!timestamp) return ''

  try {
    const date = new Date(timestamp)
    return includeTime ? date.toLocaleString() : date.toLocaleDateString()
  } catch {
    return ''
  }
}

/**
 * Sort array of tasks by their timestamp (newest first).
 *
 * Uses status timestamp as sort key. Tasks without timestamps sort to the end.
 *
 * @param tasks - Array of tasks to sort
 * @returns New sorted array
 *
 * @example
 * ```typescript
 * const sorted = sortTasksByTimestamp(tasks)
 * // Most recent tasks appear first
 * ```
 */
export function sortTasksByTimestamp(tasks: Task[]): Task[] {
  return [...tasks].sort((a, b) => {
    const timeA = a.status?.timestamp ? parseSQLiteDateTime(a.status.timestamp) : 0
    const timeB = b.status?.timestamp ? parseSQLiteDateTime(b.status.timestamp) : 0
    return timeB - timeA // Newest first
  })
}

/**
 * Filter tasks by state(s).
 *
 * Returns new array containing only tasks matching the given state(s).
 *
 * @param tasks - Array of tasks to filter
 * @param states - Single state or array of states to match
 * @returns Filtered array
 *
 * @example
 * ```typescript
 * const completed = filterTasksByState(tasks, 'completed')
 * const notRunning = filterTasksByState(tasks, ['failed', 'rejected'])
 * ```
 */
export function filterTasksByState(tasks: Task[], states: string | string[]): Task[] {
  const stateArray = Array.isArray(states) ? states : [states]
  return tasks.filter(task => stateArray.includes(task.status?.state || ''))
}

/**
 * Get task agent name from metadata.
 *
 * Extracts the agent name from task metadata, returning a default if not found.
 *
 * @param task - Task object
 * @returns Agent name or "Unknown Agent"
 *
 * @example
 * ```typescript
 * getTaskAgentName(task) // "Claude" or "Unknown Agent"
 * ```
 */
export function getTaskAgentName(task: Task): string {
  const metadata = task.metadata as Record<string, unknown> | undefined
  const name = metadata?.agent_name as string | undefined
  return name || 'Unknown Agent'
}

/**
 * Get MCP server name from task metadata.
 *
 * Extracts the MCP server name from task metadata, returning a default if not found.
 *
 * @param task - Task object
 * @returns Server name or "Unknown Server"
 *
 * @example
 * ```typescript
 * getTaskMcpServer(task) // "filesystem" or "Unknown Server"
 * ```
 */
export function getTaskMcpServer(task: Task): string {
  const metadata = task.metadata as Record<string, unknown> | undefined
  const name = metadata?.mcp_server_name as string | undefined
  return name || 'Unknown Server'
}
