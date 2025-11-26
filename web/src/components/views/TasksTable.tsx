/**
 * Tasks table component.
 *
 * Displays tasks in a table format with details button.
 *
 * @module components/views/TasksTable
 */

import React, { useCallback } from 'react'
import { CheckSquare, Clock, Eye } from 'lucide-react'
import { calculateTaskDuration } from '@/lib/utils/task-duration'
import {
  getAgentName,
  getMcpServerName,
  getConversationName,
  formatTaskStatus,
} from '@/lib/utils/task-formatting'
import { getStatusIcon } from './task-icon'
import type { Task } from '@/types/task'

interface TasksTableProps {
  tasks: Task[]
  isLoading: boolean
  hasActiveFilters: boolean
  conversationMap: Map<string, { name: string }>
  onTaskSelect: (task: Task) => void
}

/**
 * Memoized tasks table component.
 * Displays task list with status, agent, MCP server, etc.
 */
export const TasksTable = React.memo(function TasksTable({
  tasks,
  isLoading,
  hasActiveFilters,
  conversationMap,
  onTaskSelect,
}: TasksTableProps) {
  const showLoadingState = isLoading && tasks.length === 0
  const showEmptyState = tasks.length === 0 && !isLoading

  const handleTaskClick = useCallback(
    (task: Task) => {
      onTaskSelect(task)
    },
    [onTaskSelect]
  )

  if (showLoadingState) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center">
          <Clock className="w-16 h-16 text-text-secondary mx-auto mb-md animate-spin" />
          <p className="text-sm text-text-secondary">Loading tasks...</p>
        </div>
      </div>
    )
  }

  if (showEmptyState) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center">
          <CheckSquare className="w-16 h-16 text-text-secondary mx-auto mb-md" />
          <h2 className="text-lg font-heading font-semibold text-text-primary mb-sm">
            {hasActiveFilters ? 'No Tasks Match Filters' : 'No Tasks Yet'}
          </h2>
          <p className="text-sm text-text-secondary">
            {hasActiveFilters
              ? 'Try adjusting your filters to see more tasks'
              : 'Tasks will appear here when agents start processing your requests'}
          </p>
        </div>
      </div>
    )
  }

  return (
    <table className="w-full text-sm table-fixed">
      <thead className="bg-surface-variant sticky top-0 z-10">
        <tr>
          <th className="text-left p-sm font-semibold text-text-primary w-32">Status</th>
          <th className="text-left p-sm font-semibold text-text-primary w-40">Agent</th>
          <th className="text-left p-sm font-semibold text-text-primary w-40">MCP Server</th>
          <th className="text-left p-sm font-semibold text-text-primary w-40">Conversation</th>
          <th className="text-left p-sm font-semibold text-text-primary w-48">Task ID</th>
          <th className="text-left p-sm font-semibold text-text-primary w-16">Msgs</th>
          <th className="text-left p-sm font-semibold text-text-primary w-16">Arts</th>
          <th className="text-left p-sm font-semibold text-text-primary w-24">Duration</th>
          <th className="text-left p-sm font-semibold text-text-primary w-32">Timestamp</th>
          <th className="text-center p-sm font-semibold text-text-primary w-24">Actions</th>
        </tr>
      </thead>
      <tbody>
        {tasks.map((task) => {
          const messageCount = task.history?.length || 0
          const artifactCount = task.artifacts?.length || 0
          const duration = calculateTaskDuration(task)
          const statusFormatted = formatTaskStatus(task.status.state)

          return (
            <tr key={task.id} className="border-b border-surface-variant hover:bg-surface-variant/30 transition-colors">
              <td className="p-sm">
                <div className="flex items-center gap-xs">
                  {getStatusIcon(task.status.state)}
                  <span className={statusFormatted.className}>{statusFormatted.text}</span>
                </div>
              </td>
              <td className="p-sm">
                <span
                  className="text-xs text-text-secondary block overflow-hidden text-ellipsis"
                  title={getAgentName(task)}
                >
                  {getAgentName(task)}
                </span>
              </td>
              <td className="p-sm">
                <span
                  className="text-xs text-text-secondary block overflow-hidden text-ellipsis"
                  title={getMcpServerName(task)}
                >
                  {getMcpServerName(task)}
                </span>
              </td>
              <td className="p-sm">
                <span
                  className="text-xs text-text-secondary block overflow-hidden text-ellipsis"
                  title={getConversationName(task.contextId, conversationMap)}
                >
                  {getConversationName(task.contextId, conversationMap)}
                </span>
              </td>
              <td className="p-sm">
                <span
                  className="text-xs font-mono text-text-secondary block overflow-hidden text-ellipsis"
                  title={task.id}
                >
                  {task.id}
                </span>
              </td>
              <td className="p-sm text-text-secondary text-xs">{messageCount}</td>
              <td className="p-sm text-text-secondary text-xs">
                {artifactCount > 0 ? artifactCount : '-'}
              </td>
              <td className="p-sm text-text-secondary text-xs whitespace-nowrap">{duration || '-'}</td>
              <td className="p-sm text-text-secondary text-xs whitespace-nowrap">
                {task.status.timestamp ? new Date(task.status.timestamp).toLocaleDateString() : '-'}
              </td>
              <td className="p-sm text-center">
                <button
                  onClick={() => handleTaskClick(task)}
                  className="inline-flex items-center gap-xs px-sm py-xs rounded bg-primary/10 text-primary hover:bg-primary/20 transition-colors text-xs font-medium"
                >
                  <Eye className="w-4 h-4" />
                  Details
                </button>
              </td>
            </tr>
          )
        })}
      </tbody>
    </table>
  )
})
