/**
 * Tasks view component.
 *
 * Displays a filterable table of tasks with detailed modal view.
 *
 * @module components/views/TasksView
 */

import { useEffect, useState, useMemo, useCallback } from 'react'
import { useTaskStore } from '@/stores/task.store'
import { useContextStore, CONTEXT_STATE } from '@/stores/context.store'
import { useAuthStore } from '@/stores/auth.store'
import { TasksFilter } from './TasksFilter'
import { TasksTable } from './TasksTable'
import { TasksDetailsModal } from './TasksDetailsModal'
import { filterAndSortTasks, type StatusFilter, type ContextFilter } from '@/lib/utils/task-filtering'
import type { Task } from '@/types/task'

/**
 * Tasks view component.
 * Shows all tasks in the current context with filtering and detail views.
 */
export function TasksView() {
  const getAuthHeader = useAuthStore((state) => state.getAuthHeader)
  const { currentContextId } = useContextStore()
  const { byId, allIds, isLoading, fetchTasksByContext } = useTaskStore()
  const conversationMap = useContextStore((state) => state.conversations)

  const [statusFilter, setStatusFilter] = useState<StatusFilter>('all')
  const [contextFilter, setContextFilter] = useState<ContextFilter>('current')
  const [selectedTask, setSelectedTask] = useState<Task | null>(null)

  useEffect(() => {
    const authHeader = getAuthHeader()
    if (!authHeader || !currentContextId || currentContextId === CONTEXT_STATE.LOADING) return

    fetchTasksByContext(currentContextId, authHeader)
  }, [getAuthHeader, currentContextId, fetchTasksByContext])

  const allTasks = useMemo(
    () => allIds.map(id => byId[id]).filter(Boolean),
    [allIds, byId]
  )

  const filteredTasks = useMemo(
    () => filterAndSortTasks(allTasks, statusFilter, contextFilter, currentContextId),
    [allTasks, statusFilter, contextFilter, currentContextId]
  )

  const handleStatusFilterChange = useCallback((filter: StatusFilter) => {
    setStatusFilter(filter)
  }, [])

  const handleContextFilterChange = useCallback((filter: ContextFilter) => {
    setContextFilter(filter)
  }, [])

  const handleClearFilters = useCallback(() => {
    setStatusFilter('all')
    setContextFilter('all')
  }, [])

  const handleSelectTask = useCallback((task: Task) => {
    setSelectedTask(task)
  }, [])

  const handleCloseModal = useCallback(() => {
    setSelectedTask(null)
  }, [])

  return (
    <div className="h-full overflow-auto flex flex-col">
      <TasksFilter
        statusFilter={statusFilter}
        onStatusFilterChange={handleStatusFilterChange}
        contextFilter={contextFilter}
        onContextFilterChange={handleContextFilterChange}
        onClearFilters={handleClearFilters}
      />

      <div className="flex-1 overflow-auto">
        <TasksTable
          tasks={filteredTasks}
          isLoading={isLoading}
          hasActiveFilters={statusFilter !== 'all' || contextFilter !== 'all'}
          conversationMap={conversationMap}
          onTaskSelect={handleSelectTask}
        />
      </div>

      <TasksDetailsModal
        task={selectedTask}
        onClose={handleCloseModal}
        conversationMap={conversationMap}
      />
    </div>
  )
}
