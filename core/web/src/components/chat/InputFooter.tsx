/**
 * Input footer with attach button and keyboard hints.
 *
 * Shows file attachment button and keyboard shortcuts (desktop only).
 *
 * @module chat/InputFooter
 */

import React from 'react'
import { Paperclip } from 'lucide-react'
import { cn } from '@/lib/utils/cn'

interface InputFooterProps {
  disabled?: boolean
  isStreaming?: boolean
  onAttachClick: () => void
  fileInputRef: React.RefObject<HTMLInputElement | null>
}

export const InputFooter = React.memo(function InputFooter({
  disabled,
  isStreaming,
  onAttachClick,
  fileInputRef,
}: InputFooterProps) {
  return (
    <div className="hidden md:block mt-sm pt-sm border-t border-primary/10">
      <div className="flex items-center justify-between text-xs text-text-secondary font-body">
        <div className="flex items-center gap-md">
          <button
            onClick={onAttachClick}
            disabled={disabled || isStreaming}
            className={cn('flex items-center gap-xs transition-colors duration-fast', 'hover:text-primary', 'disabled:opacity-50 disabled:cursor-not-allowed')}
            aria-label="Attach files"
          >
            <Paperclip className="w-3.5 h-3.5" />
            <span>Attach</span>
          </button>

          <input ref={fileInputRef} type="file" multiple className="hidden" disabled={disabled || isStreaming} />

          <span className="flex items-center gap-xs">
            <kbd className="px-xs py-0.5 rounded-sm bg-surface-variant text-text-secondary font-mono text-xs border border-primary/10">⏎</kbd>
            <span>Send</span>
          </span>
          <span className="flex items-center gap-xs">
            <kbd className="px-xs py-0.5 rounded-sm bg-surface-variant text-text-secondary font-mono text-xs border border-primary/10">⇧⏎</kbd>
            <span>New line</span>
          </span>
        </div>
      </div>
    </div>
  )
})
