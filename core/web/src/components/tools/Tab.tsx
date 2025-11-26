/**
 * Tab component for execution tabs.
 *
 * Renders a single tab button with active/inactive styling.
 *
 * @module tools/Tab
 */

import React from 'react'
import { cn } from '@/lib/utils/cn'

interface TabProps {
  label: string
  isActive: boolean
  onClick: () => void
}

export const Tab = React.memo(function Tab({ label, isActive, onClick }: TabProps) {
  return (
    <button
      onClick={onClick}
      className={cn(
        'px-4 py-2 text-sm font-medium border-b-2 transition-colors',
        isActive
          ? 'border-primary text-primary'
          : 'border-transparent text-muted hover:text-foreground'
      )}
    >
      {label}
    </button>
  )
})
