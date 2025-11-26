/**
 * ModalHeader component.
 *
 * Displays modal header with optional title and close button.
 *
 * @module ui/ModalHeader
 */

import React from 'react'
import { X } from 'lucide-react'
import { cn } from '@/lib/utils/cn'
import { Button } from './Button'

interface ModalHeaderProps {
  title?: string
  titleId?: string
  onClose?: () => void
  showCloseButton?: boolean
  isMobile?: boolean
  children?: React.ReactNode
  className?: string
}

export const ModalHeader = React.memo(function ModalHeader({
  title,
  titleId,
  onClose,
  showCloseButton = true,
  isMobile = false,
  children,
  className,
}: ModalHeaderProps) {
  if (children) {
    return (
      <div className={cn('p-md border-b border-primary/10', className)}>{children}</div>
    )
  }

  if (title) {
    return (
      <div
        className={cn(
          'flex items-center justify-between border-b border-primary/10',
          isMobile ? 'p-sm' : 'p-md'
        )}
      >
        <h2
          id={titleId}
          className={cn(
            'font-heading font-semibold text-text-primary',
            isMobile ? 'text-lg' : 'text-xl'
          )}
        >
          {title}
        </h2>
        {showCloseButton && onClose && (
          <Button
            variant="ghost"
            size="sm"
            icon={X}
            onClick={onClose}
            aria-label="Close modal"
          />
        )}
      </div>
    )
  }

  return null
})
