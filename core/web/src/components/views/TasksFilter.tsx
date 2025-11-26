/**
 * Task filter controls component.
 *
 * Provides status and context filtering for the tasks view.
 *
 * @module components/views/TasksFilter
 */

import React, { useCallback } from 'react'
import { X } from 'lucide-react'
import { cn } from '@/lib/utils/cn'
import { hasActiveFilters, type StatusFilter, type ContextFilter } from '@/lib/utils/task-filtering'

interface TasksFilterProps {
  statusFilter: StatusFilter
  onStatusFilterChange: (filter: StatusFilter) => void
  contextFilter: ContextFilter
  onContextFilterChange: (filter: ContextFilter) => void
  onClearFilters: () => void
}

/**
 * Memoized tasks filter component.
 * Provides toggle buttons for status and context filtering.
 */
export const TasksFilter = React.memo(function TasksFilter({
  statusFilter,
  onStatusFilterChange,
  contextFilter,
  onContextFilterChange,
  onClearFilters,
}: TasksFilterProps) {
  const isFiltered = hasActiveFilters(statusFilter, contextFilter)

  const handleStatusClick = useCallback(
    (filter: StatusFilter) => {
      if (statusFilter === filter) {
        onStatusFilterChange('all')
      } else {
        onStatusFilterChange(filter)
      }
    },
    [statusFilter, onStatusFilterChange]
  )

  const handleContextClick = useCallback(
    (filter: ContextFilter) => {
      if (contextFilter === filter) {
        onContextFilterChange('all')
      } else {
        onContextFilterChange(filter)
      }
    },
    [contextFilter, onContextFilterChange]
  )

  return (
    <div className="flex-shrink-0 flex flex-wrap items-center gap-sm p-sm border-b border-surface-variant bg-surface">
      {/* Status filters */}
      <div className="flex gap-xs">
        <button
          onClick={() => onStatusFilterChange('all')}
          className={cn(
            'px-sm py-xs rounded text-xs font-medium transition-colors',
            statusFilter === 'all'
              ? 'bg-primary text-white'
              : 'bg-surface-variant text-text-secondary hover:bg-surface-variant/80'
          )}
        >
          All
        </button>
        <button
          onClick={() => handleStatusClick('working')}
          className={cn(
            'px-sm py-xs rounded text-xs font-medium transition-colors flex items-center gap-xs',
            statusFilter === 'working'
              ? 'bg-warning text-white'
              : 'bg-surface-variant text-text-secondary hover:bg-surface-variant/80'
          )}
        >
          Working
          {statusFilter === 'working' && <X className="w-3 h-3" />}
        </button>
        <button
          onClick={() => handleStatusClick('completed')}
          className={cn(
            'px-sm py-xs rounded text-xs font-medium transition-colors flex items-center gap-xs',
            statusFilter === 'completed'
              ? 'bg-success text-white'
              : 'bg-surface-variant text-text-secondary hover:bg-surface-variant/80'
          )}
        >
          Completed
          {statusFilter === 'completed' && <X className="w-3 h-3" />}
        </button>
        <button
          onClick={() => handleStatusClick('failed')}
          className={cn(
            'px-sm py-xs rounded text-xs font-medium transition-colors flex items-center gap-xs',
            statusFilter === 'failed'
              ? 'bg-error text-white'
              : 'bg-surface-variant text-text-secondary hover:bg-surface-variant/80'
          )}
        >
          Failed
          {statusFilter === 'failed' && <X className="w-3 h-3" />}
        </button>
      </div>

      {/* Divider */}
      <div className="h-4 w-px bg-surface-variant" />

      {/* Context filters */}
      <div className="flex gap-xs">
        <button
          onClick={() => onContextFilterChange('all')}
          className={cn(
            'px-sm py-xs rounded text-xs font-medium transition-colors',
            contextFilter === 'all'
              ? 'bg-primary text-white'
              : 'bg-surface-variant text-text-secondary hover:bg-surface-variant/80'
          )}
        >
          All Conversations
        </button>
        <button
          onClick={() => handleContextClick('current')}
          className={cn(
            'px-sm py-xs rounded text-xs font-medium transition-colors flex items-center gap-xs',
            contextFilter === 'current'
              ? 'bg-primary text-white'
              : 'bg-surface-variant text-text-secondary hover:bg-surface-variant/80'
          )}
        >
          Current
          {contextFilter === 'current' && <X className="w-3 h-3" />}
        </button>
      </div>

      {/* Clear filters */}
      <div className="flex gap-xs ml-auto">
        {isFiltered && (
          <button
            onClick={onClearFilters}
            className="px-sm py-xs rounded text-xs font-medium bg-error/10 text-error hover:bg-error/20 transition-colors flex items-center gap-xs"
          >
            Clear All
            <X className="w-3 h-3" />
          </button>
        )}
      </div>
    </div>
  )
})
