/**
 * Task details modal component.
 *
 * Shows detailed information about a selected task.
 *
 * @module components/views/TasksDetailsModal
 */

import React, { useCallback } from 'react'
import { X, Package } from 'lucide-react'
import { Card } from '@/components/ui/Card'
import { calculateTaskDuration } from '@/lib/utils/task-duration'
import { formatTaskStatus, getConversationName } from '@/lib/utils/task-formatting'
import { getStatusIcon } from './task-icon'
import type { Task } from '@/types/task'
import type { Message1, Part, Artifact } from '@a2a-js/sdk'

interface TasksDetailsModalProps {
  task: Task | null
  onClose: () => void
  conversationMap: Map<string, { name: string }>
}

/**
 * Memoized task details modal component.
 * Displays full task information in a modal.
 */
export const TasksDetailsModal = React.memo(function TasksDetailsModal({
  task,
  onClose,
  conversationMap,
}: TasksDetailsModalProps) {
  const handleBackdropClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === e.currentTarget) {
        onClose()
      }
    },
    [onClose]
  )

  if (!task) return null

  const statusFormatted = formatTaskStatus(task.status.state)
  const duration = calculateTaskDuration(task)

  return (
    <div
      className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-md"
      onClick={handleBackdropClick}
    >
      <div onClick={(e) => e.stopPropagation()}>
        <Card
          variant="accent"
          padding="lg"
          elevation="lg"
          className="max-w-4xl w-full max-h-[90vh] overflow-auto"
        >
          <div className="space-y-md">
            {/* Header */}
            <div className="flex items-start justify-between gap-md mb-md">
              <div className="flex-1">
                <h2 className="text-lg font-heading font-bold text-text-primary mb-xs">
                  Task Details
                </h2>
                <p className="text-xs font-mono text-text-secondary">{task.id}</p>
              </div>
              <button
                onClick={onClose}
                className="p-xs rounded hover:bg-surface-variant transition-colors"
              >
                <X className="w-5 h-5 text-text-secondary" />
              </button>
            </div>

            {/* Status and Metadata */}
            <div className="grid gap-sm">
              <div className="flex items-center gap-sm">
                <span className="text-sm font-semibold text-text-primary">Status:</span>
                <div className="flex items-center gap-sm">
                  {getStatusIcon(task.status.state)}
                  <span className={statusFormatted.className}>{statusFormatted.text}</span>
                </div>
              </div>

              {task.status.timestamp && (
                <div>
                  <span className="text-sm font-semibold text-text-primary">Timestamp: </span>
                  <span className="text-sm text-text-secondary">
                    {new Date(task.status.timestamp).toLocaleString()}
                  </span>
                </div>
              )}

              {duration && (
                <div>
                  <span className="text-sm font-semibold text-text-primary">Duration: </span>
                  <span className="text-sm text-text-secondary">{duration}</span>
                </div>
              )}

              <div>
                <span className="text-sm font-semibold text-text-primary">Conversation: </span>
                <span className="text-sm text-text-secondary">
                  {getConversationName(task.contextId, conversationMap)}
                </span>
              </div>
            </div>

            {/* Message History */}
            {task.history && task.history.length > 0 && (
              <div>
                <h3 className="text-sm font-semibold text-text-primary mb-sm">
                  Message History ({task.history.length})
                </h3>
                <div className="space-y-sm">
                  {task.history.map((message: Message1, idx: number) => {
                    const messageKey = message.messageId || `${task.id}-msg-${idx}-${message.role}`
                    return (
                      <div key={messageKey} className="bg-surface-variant/50 rounded p-sm">
                        <div className="font-medium capitalize text-xs text-text-primary mb-xs">
                          {message.role}
                        </div>
                        <div className="text-xs text-text-secondary">
                          {message.parts.map((part: Part, partIdx: number) => {
                            if ('text' in part) {
                              const partKey = `${messageKey}-part-${partIdx}`
                              return (
                                <div key={partKey} className="whitespace-pre-wrap">
                                  {part.text}
                                </div>
                              )
                            }
                            return null
                          })}
                        </div>
                      </div>
                    )
                  })}
                </div>
              </div>
            )}

            {/* Artifacts */}
            {task.artifacts && task.artifacts.length > 0 && (
              <div>
                <h3 className="text-sm font-semibold text-text-primary mb-sm flex items-center gap-xs">
                  <Package className="w-4 h-4" />
                  Artifacts ({task.artifacts.length})
                </h3>
                <div className="space-y-xs">
                  {task.artifacts.map((artifact: Artifact) => {
                    const artifactKey = artifact.artifactId || `${task.id}-art-${artifact.name}`
                    return (
                      <div key={artifactKey} className="text-xs text-text-secondary bg-surface-variant/50 rounded p-xs">
                        <div className="font-medium">{artifact.name || artifact.artifactId}</div>
                        {artifact.description && (
                          <div className="text-xs mt-xs">{artifact.description}</div>
                        )}
                      </div>
                    )
                  })}
                </div>
              </div>
            )}

            {/* Metadata */}
            {task.metadata && (
              <div>
                <h3 className="text-sm font-semibold text-text-primary mb-sm">Metadata</h3>
                <pre className="text-xs bg-surface-variant/50 rounded p-sm overflow-auto max-h-60">
                  {JSON.stringify(task.metadata, null, 2)}
                </pre>
              </div>
            )}
          </div>
        </Card>
      </div>
    </div>
  )
})
