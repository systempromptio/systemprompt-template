import React from 'react'
import { formatDate } from '@/lib/utils/format'

interface TaskTimestampProps {
  createdAt: string
}

export const TaskTimestamp = React.memo(function TaskTimestamp({
  createdAt,
}: TaskTimestampProps) {
  return (
    <span className="text-text-secondary">
      {formatDate(createdAt)}
    </span>
  )
})
