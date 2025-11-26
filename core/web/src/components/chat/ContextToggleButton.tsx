/**
 * ContextToggleButton component.
 *
 * Button to open/close conversation selector for authenticated users.
 *
 * @module chat/ContextToggleButton
 */

import React from 'react'
import { MessageSquare, ChevronDown } from 'lucide-react'

interface ContextToggleButtonProps {
  email?: string
  conversationName: string
  isOpen: boolean
  onClick: () => void
  'aria-haspopup'?: 'listbox'
  'aria-expanded'?: boolean
  'aria-label'?: string
}

export const ContextToggleButton = React.memo(function ContextToggleButton({
  email,
  conversationName,
  isOpen,
  onClick,
  'aria-haspopup': ariaHasPopup,
  'aria-expanded': ariaExpanded,
  'aria-label': ariaLabel,
}: ContextToggleButtonProps) {
  return (
    <button
      onClick={onClick}
      className="flex items-center gap-sm px-sm py-sm text-sm hover:bg-primary/5 rounded-lg transition-fast"
      title="Manage conversations"
      aria-haspopup={ariaHasPopup}
      aria-expanded={ariaExpanded}
      aria-label={ariaLabel}
    >
      <MessageSquare className="w-4 h-4 text-primary" aria-hidden="true" />
      <div className="flex flex-col items-start">
        <span className="text-xs text-text-secondary font-body">Welcome {email?.split('@')[0]}</span>
        <span className="text-text-primary font-heading uppercase tracking-wide max-w-[150px] truncate">
          {conversationName}
        </span>
      </div>
      <ChevronDown className={`w-4 h-4 text-primary transition-transform ${isOpen ? 'rotate-180' : ''}`} aria-hidden="true" />
    </button>
  )
})
