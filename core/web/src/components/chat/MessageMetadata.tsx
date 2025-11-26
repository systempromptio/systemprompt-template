/**
 * Message metadata component.
 *
 * Displays timestamp, task info, and metadata for messages.
 *
 * @module components/chat/MessageMetadata
 */

import React, { useMemo } from 'react'
import { formatDate } from '@/lib/utils/format'
import { calculateTaskDuration } from '@/lib/utils/task-duration'
import { Clock, BookOpen } from 'lucide-react'
import { cn } from '@/lib/utils/cn'
import { useSkillStore } from '@/stores/skill.store'
import type { ChatMessage } from '@/stores/chat.store'

interface MessageMetadataProps {
  message: ChatMessage
  isUser: boolean
}

/**
 * Memoized message metadata component.
 * Shows timestamp and task details.
 */
export const MessageMetadata = React.memo(function MessageMetadata({
  message,
  isUser,
}: MessageMetadataProps) {
  const taskDuration = useMemo(
    () => message.task ? calculateTaskDuration(message.task) : null,
    [message.task]
  )

  const usedSkills = useMemo(() => {
    if (!message.contextId || !message.task?.id) {
      console.log('[SKILL DEBUG] MessageMetadata - missing contextId or taskId:', {
        hasContextId: !!message.contextId,
        hasTaskId: !!message.task?.id,
        contextId: message.contextId,
        taskId: message.task?.id
      })
      return []
    }

    const storeState = useSkillStore.getState()
    const skillIds = storeState.byContext[message.contextId]?.[message.task.id]

    console.log('[SKILL DEBUG] MessageMetadata - checking for skills:', {
      contextId: message.contextId,
      taskId: message.task.id,
      skillIds: skillIds,
      allContexts: Object.keys(storeState.byContext),
      contextData: storeState.byContext[message.contextId],
      fullStore: storeState.byContext
    })

    return skillIds || []
  }, [message.contextId, message.task?.id])

  const openSkill = useSkillStore((state) => state.openSkill)

  return (
    <div className={cn(
      'text-xs mt-xs font-body w-full space-y-xs',
      isUser ? 'text-right' : ''
    )}>
      <span className={cn(isUser ? 'text-text-secondary/80' : 'text-text-secondary')}>
        {formatDate(message.timestamp)}
      </span>

      {!isUser && message.task && (
        <div className="mt-xs flex items-center gap-sm text-xs flex-wrap">
          <span className="text-text-secondary/60">
            Task: {message.task.id.slice(0, 8)}
          </span>

          {taskDuration && (
            <div className="inline-flex items-center gap-xs text-success">
              <Clock className="w-3 h-3" />
              <span>Completed in {taskDuration}</span>
            </div>
          )}
        </div>
      )}

      {!isUser && usedSkills.length > 0 && (
        <button
          onClick={() => usedSkills.length > 0 && openSkill(usedSkills[0])}
          className="inline-flex items-center gap-xs px-md py-xs bg-purple-50 hover:bg-purple-100 border border-purple-200 rounded-lg text-sm text-purple-700 transition-colors cursor-pointer dark:bg-purple-900/30 dark:hover:bg-purple-900/50 dark:border-purple-800 dark:text-purple-300"
        >
          <BookOpen className="w-4 h-4" />
          <span>
            Used {usedSkills.length} skill{usedSkills.length !== 1 ? 's' : ''}
          </span>
        </button>
      )}
    </div>
  )
})
