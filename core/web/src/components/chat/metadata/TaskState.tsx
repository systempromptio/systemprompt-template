import React from 'react'
import { cn } from '@/lib/utils/cn'
import { getStateInfo } from '../task/taskStateUtils'
import type { TaskState as TaskStateType } from '@a2a-js/sdk'

interface TaskStateProps {
  state: TaskStateType
}

export const TaskState = React.memo(function TaskState({ state }: TaskStateProps) {
  const stateInfo = getStateInfo(state)

  return (
    <>
      <span className={cn('inline-flex items-center gap-1', stateInfo.textColor)}>
        <span className={stateInfo.iconColor}>{stateInfo.icon}</span>
        {stateInfo.label}
      </span>
      <span className="text-text-secondary/40">·</span>
    </>
  )
})
