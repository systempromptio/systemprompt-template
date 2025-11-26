/**
 * Task duration calculation utilities.
 *
 * Provides human-readable duration formatting for completed/failed tasks.
 *
 * @module lib/utils/task-duration
 */

import type { Task } from '@/types/task'

/**
 * Parses SQLite and RFC3339 date formats to milliseconds since epoch.
 *
 * @param dateStr - Date string in SQLite format (YYYY-MM-DD HH:MM:SS) or RFC3339 (YYYY-MM-DDTHH:MM:SS.sssZ)
 * @returns Milliseconds since epoch
 */
function parseSQLiteDateTime(dateStr: string): number {
  if (dateStr.includes('T')) {
    return new Date(dateStr).getTime()
  } else {
    return new Date(dateStr + 'Z').getTime()
  }
}

/**
 * Calculates human-readable duration for a task.
 *
 * Handles multiple date sources with graceful fallback:
 * 1. execution_time_ms metadata field (preferred)
 * 2. started_at/completed_at timestamps
 * 3. created_at/status.timestamp as fallback
 *
 * @param task - Task with metadata and status
 * @returns Formatted duration (e.g., "1h 30m", "45s", "123ms") or null if not completed
 *
 * @example
 * ```typescript
 * const duration = calculateTaskDuration(task)
 * // Returns "5m 23s" or null if still in progress
 * ```
 */
export function calculateTaskDuration(task: Task): string | null {
  if (task.status.state !== 'completed' && task.status.state !== 'failed') {
    return null
  }

  let durationMs: number

  if (task.metadata.execution_time_ms !== undefined && task.metadata.execution_time_ms !== null) {
    durationMs = task.metadata.execution_time_ms
  } else if (task.metadata.started_at && task.metadata.completed_at) {
    const startTime = parseSQLiteDateTime(task.metadata.started_at)
    const endTime = parseSQLiteDateTime(task.metadata.completed_at)

    if (isNaN(startTime) || isNaN(endTime)) {
      return null
    }

    durationMs = endTime - startTime
  } else if (task.status.timestamp && task.metadata.created_at) {
    const startTime = parseSQLiteDateTime(task.metadata.created_at)
    const endTime = parseSQLiteDateTime(task.status.timestamp)

    if (isNaN(startTime) || isNaN(endTime)) {
      return null
    }

    durationMs = endTime - startTime
  } else {
    return null
  }

  const seconds = Math.floor(durationMs / 1000)
  const milliseconds = durationMs % 1000

  if (durationMs < 1000) {
    return `${durationMs}ms`
  } else if (seconds < 1) {
    return `${seconds}.${Math.floor(milliseconds / 100)}s`
  } else if (seconds < 10) {
    return `${seconds}.${Math.floor(milliseconds / 100)}s`
  } else if (seconds < 60) {
    return `${seconds}s`
  } else if (seconds < 3600) {
    const minutes = Math.floor(seconds / 60)
    const remainingSeconds = seconds % 60
    return remainingSeconds > 0 ? `${minutes}m ${remainingSeconds}s` : `${minutes}m`
  } else {
    const hours = Math.floor(seconds / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    return minutes > 0 ? `${hours}h ${minutes}m` : `${hours}h`
  }
}
