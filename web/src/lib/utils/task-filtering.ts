/**
 * Task filtering and sorting utilities.
 *
 * Provides helpers for filtering tasks by status and context.
 *
 * @module lib/utils/task-filtering
 */

import type { Task } from '@/types/task'

export type StatusFilter = 'all' | 'submitted' | 'working' | 'completed' | 'failed'
export type ContextFilter = 'all' | 'current'

/**
 * Filters and sorts tasks based on status and context.
 *
 * @param tasks - Array of tasks to filter
 * @param statusFilter - Filter by status state
 * @param contextFilter - Filter by context
 * @param currentContextId - Current context ID
 * @returns Filtered and sorted task array
 */
export function filterAndSortTasks(
  tasks: Task[],
  statusFilter: StatusFilter,
  contextFilter: ContextFilter,
  currentContextId: string | null
): Task[] {
  return tasks
    .filter((task) => {
      if (statusFilter !== 'all' && task.status.state !== statusFilter) {
        return false
      }
      if (contextFilter === 'current' && task.contextId !== currentContextId) {
        return false
      }
      return true
    })
    .sort((a, b) => {
      const timeA = a.status.timestamp ? new Date(a.status.timestamp).getTime() : 0
      const timeB = b.status.timestamp ? new Date(b.status.timestamp).getTime() : 0
      return timeB - timeA
    })
}

/**
 * Checks if any filters are active.
 *
 * @param statusFilter - Status filter value
 * @param contextFilter - Context filter value
 * @returns True if either filter is not 'all'
 */
export function hasActiveFilters(statusFilter: StatusFilter, contextFilter: ContextFilter): boolean {
  return statusFilter !== 'all' || contextFilter !== 'all'
}
