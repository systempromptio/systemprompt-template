/**
 * File preview component for message input.
 *
 * Shows list of attached files with remove buttons.
 *
 * @module chat/FilePreview
 */

import React from 'react'
import { Paperclip, X } from 'lucide-react'
import { cn } from '@/lib/utils/cn'
import { formatFileSize } from '@/lib/utils/format'

interface FilePreviewProps {
  files: File[]
  onRemove: (index: number) => void
}

export const FilePreview = React.memo(function FilePreview({ files, onRemove }: FilePreviewProps) {
  if (files.length === 0) return null

  return (
    <div className="px-lg pt-md pb-sm">
      <div className="flex flex-wrap gap-sm">
        {files.map((file, index) => (
          <div
            key={index}
            className={cn(
              'flex items-center gap-sm rounded-md px-sm py-xs text-sm font-body',
              'bg-surface-variant border border-primary/10',
              'hover:bg-surface-variant/80 transition-all duration-fast'
            )}
          >
            <Paperclip className="w-3.5 h-3.5 text-primary" />
            <span className="font-medium text-text-primary">{file.name}</span>
            <span className="text-text-secondary text-xs">({formatFileSize(file.size)})</span>
            <button
              onClick={() => onRemove(index)}
              className="ml-xs p-xs hover:bg-error/10 text-text-secondary hover:text-error rounded transition-fast"
              aria-label={`Remove ${file.name}`}
            >
              <X className="w-3 h-3" />
            </button>
          </div>
        ))}
      </div>
    </div>
  )
})
