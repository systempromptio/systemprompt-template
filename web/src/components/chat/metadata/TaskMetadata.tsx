import React from 'react'
import { Package } from 'lucide-react'
import { TaskTimestamp } from './TaskTimestamp'
import { TaskSkillBadges } from './TaskSkillBadges'
import { TaskState } from './TaskState'
import { TaskDuration } from './TaskDuration'
import { TaskExecutionSteps } from './TaskExecutionSteps'
import { useUIStateStore } from '@/stores/ui-state.store'
import type { Task } from '@/types/task'

interface TaskMetadataProps {
  task: Task
  contextId: string
  artifactCount?: number
  onArtifactClick?: () => void
}

export const TaskMetadata = React.memo(function TaskMetadata({
  task,
  contextId,
  artifactCount = 0,
  onArtifactClick,
}: TaskMetadataProps) {
  const activeStreamingTaskId = useUIStateStore((s) => s.activeStreamingTaskId)
  const isStreaming = activeStreamingTaskId === task.id

  if (isStreaming) return null

  const showSkills = true
  const showArtifacts = artifactCount > 0 && onArtifactClick !== undefined
  const showTaskInfo = true

  return (
    <div className="text-xs mt-xs font-body w-full">
      <div className="flex items-center gap-sm flex-wrap">
        {showSkills && <TaskSkillBadges taskId={task.id} contextId={contextId} />}

        {showArtifacts && (
          <>
            <button
              onClick={onArtifactClick}
              className="inline-flex items-center gap-xs px-2 py-0.5 rounded-full bg-primary/10 text-primary hover:bg-primary/20 transition-colors"
            >
              <Package className="w-3 h-3" />
              <span>
                {artifactCount} artifact{artifactCount !== 1 ? 's' : ''}
              </span>
            </button>
            <span className="text-text-secondary/40">·</span>
          </>
        )}

        {showTaskInfo && <TaskState state={task.status.state} />}

        <TaskTimestamp createdAt={task.metadata.created_at} />

        {showTaskInfo && (
          <>
            <span className="text-text-secondary/40">·</span>
            <span className="text-text-secondary/60">{task.id.slice(0, 8)}</span>
            <TaskDuration task={task} />
          </>
        )}

        <TaskExecutionSteps taskId={task.id} />
      </div>
    </div>
  )
})
