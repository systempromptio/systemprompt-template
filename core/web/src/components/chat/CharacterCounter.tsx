/**
 * Character counter for message input.
 *
 * Shows character count with warning when near limit.
 *
 * @module chat/CharacterCounter
 */

import React from 'react'
import { cn } from '@/lib/utils/cn'

interface CharacterCounterProps {
  charCount: number
  isVisible: boolean
  isNearLimit: boolean
}

export const CharacterCounter = React.memo(function CharacterCounter({
  charCount,
  isVisible,
  isNearLimit,
}: CharacterCounterProps) {
  if (!isVisible) return null

  return (
    <div className="absolute right-14 top-3">
      <span
        className={cn(
          'text-xs font-mono transition-colors duration-fast px-xs py-xs rounded',
          isNearLimit ? 'text-warning font-semibold bg-warning/10' : 'text-text-secondary'
        )}
      >
        {charCount}
      </span>
    </div>
  )
})
