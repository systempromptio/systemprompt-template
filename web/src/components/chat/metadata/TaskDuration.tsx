import React, { useMemo } from 'react'
import { Clock } from 'lucide-react'
import { calculateTaskDuration } from '@/lib/utils/task-duration'
import { formatTokenInfo } from './types'
import type { Task } from '@/types/task'

interface TaskDurationProps {
  task: Task
}

export const TaskDuration = React.memo(function TaskDuration({ task }: TaskDurationProps) {
  const duration = useMemo(() => calculateTaskDuration(task), [task])

  const tokenInfo = useMemo(() => {
    const metadata = task.metadata
    return formatTokenInfo(metadata.input_tokens, metadata.output_tokens)
  }, [task.metadata])

  if (!duration && !tokenInfo) return null

  return (
    <>
      <span className="text-text-secondary/40">·</span>
      <span className="inline-flex items-center gap-xs text-success">
        <Clock className="w-3 h-3" />
        {duration && `${duration}`}
        {duration && tokenInfo && ' · '}
        {tokenInfo?.formatted}
      </span>
    </>
  )
})
