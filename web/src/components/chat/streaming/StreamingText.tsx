/**
 * Text content renderer with markdown support.
 *
 * Renders message text content with markdown formatting.
 */

import { lazy, Suspense } from 'react'
import { cn } from '@/lib/utils/cn'

const MarkdownContent = lazy(() =>
  import('@/components/markdown/MarkdownContent').then((m) => ({ default: m.MarkdownContent }))
)

interface StreamingTextProps {
  text: string
  isUser?: boolean
  isStreaming?: boolean
}

export function StreamingText({ text, isUser = false }: StreamingTextProps) {
  const hasContent = text.trim().length > 0

  return (
    <div className={cn('animate-fadeIn', isUser && 'text-white')}>
      {hasContent && (
        <Suspense fallback={<div className="text-text-secondary">{text}</div>}>
          <MarkdownContent content={text || ''} />
        </Suspense>
      )}
    </div>
  )
}
