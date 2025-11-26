import React, { useState } from 'react'
import { ChevronDown, ChevronRight } from 'lucide-react'
import { cn } from '@/lib/utils/cn'
import type { Task } from '@a2a-js/sdk'
import { getStateInfo, extractMessageText } from './taskStateUtils'

interface TaskStateIndicatorProps {
  task: Task
  showHistory?: boolean
  compact?: boolean
  onInputRequired?: () => void
  onAuthRequired?: () => void
}

export const TaskStateIndicator = React.memo(function TaskStateIndicator({
  task,
  showHistory = false,
  compact = false,
  onInputRequired,
  onAuthRequired,
}: TaskStateIndicatorProps) {
  const [isExpanded, setIsExpanded] = useState(false)

  const stateInfo = getStateInfo(task.status.state)

  if (compact) {
    return (
      <div className="flex items-center gap-2">
        <div className={cn('transition-colors', stateInfo.iconColor)}>
          {stateInfo.icon}
        </div>
        <span className={cn('text-sm font-medium', stateInfo.textColor)}>
          {stateInfo.label}
        </span>
      </div>
    )
  }

  return (
    <div className="space-y-2">
      <div
        className={cn(
          'flex items-center gap-3 p-3 rounded-lg border transition-all',
          stateInfo.bgColor,
          stateInfo.borderColor
        )}
      >
        <div
          key={task.status.state}
          className={cn(
            'animate-scaleIn transition-colors',
            stateInfo.iconColor
          )}
        >
          {stateInfo.icon}
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className={cn('text-sm font-semibold', stateInfo.textColor)}>
              {stateInfo.label}
            </span>
            {task.status.timestamp && (
              <span className="text-xs text-gray-500">
                {new Date(task.status.timestamp).toLocaleTimeString()}
              </span>
            )}
          </div>

          {task.status.message && (
            <div className="text-xs text-gray-600 mt-1 line-clamp-2">
              {extractMessageText(task.status.message)}
            </div>
          )}
        </div>

        {showHistory && (
          <button
            onClick={() => setIsExpanded(!isExpanded)}
            className="p-1 hover:bg-white/50 rounded transition-colors"
          >
            {isExpanded ? (
              <ChevronDown className="w-4 h-4 text-gray-600" />
            ) : (
              <ChevronRight className="w-4 h-4 text-gray-600" />
            )}
          </button>
        )}

        {task.status.state === 'input-required' && onInputRequired && (
          <button
            onClick={onInputRequired}
            className="px-3 py-1.5 text-xs font-medium bg-amber-500 text-white rounded hover:bg-amber-600 transition-colors"
          >
            Respond
          </button>
        )}

        {task.status.state === 'auth-required' && onAuthRequired && (
          <button
            onClick={onAuthRequired}
            className="px-3 py-1.5 text-xs font-medium bg-purple-500 text-white rounded hover:bg-purple-600 transition-colors"
          >
            Authorize
          </button>
        )}
      </div>

      {isExpanded && task.history && task.history.length > 0 && (
        <div className="ml-6 pl-4 border-l-2 border-gray-200 space-y-2 animate-fadeIn">
          <div className="text-xs font-medium text-gray-500 uppercase tracking-wider">
            History
          </div>
          {task.history.map((message, idx) => (
            <div
              key={idx}
              className="text-sm text-gray-600 p-2 bg-gray-50 rounded"
            >
              <div className="font-medium text-gray-700 mb-1">
                {message.role === 'user' ? 'User' : 'Agent'}
              </div>
              <div className="text-xs">
                {extractMessageText(message)}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
})
