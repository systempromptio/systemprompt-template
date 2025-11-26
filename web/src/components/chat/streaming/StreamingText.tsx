import { lazy, Suspense } from 'react'
import { cn } from '@/lib/utils/cn'

const MarkdownContent = lazy(() => import('@/components/markdown/MarkdownContent').then(m => ({ default: m.MarkdownContent })))

interface StreamingTextProps {
  text: string
  isStreaming?: boolean
  isUser?: boolean
}

export function StreamingText({ text, isStreaming = false, isUser = false }: StreamingTextProps) {
  // Show loading dots only when streaming AND no content yet
  const showLoadingDots = isStreaming && !text.trim()

  return (
    <div className="relative">
      {showLoadingDots ? (
        <div className="flex items-center gap-1 py-2">
          <span className={cn(
            'w-2 h-2 rounded-full animate-bounce',
            isUser ? 'bg-white/60' : 'bg-primary/60'
          )} style={{ animationDelay: '0ms' }} />
          <span className={cn(
            'w-2 h-2 rounded-full animate-bounce',
            isUser ? 'bg-white/60' : 'bg-primary/60'
          )} style={{ animationDelay: '150ms' }} />
          <span className={cn(
            'w-2 h-2 rounded-full animate-bounce',
            isUser ? 'bg-white/60' : 'bg-primary/60'
          )} style={{ animationDelay: '300ms' }} />
        </div>
      ) : (
        <div className={cn('animate-fadeIn', isUser && 'text-white')}>
          <Suspense fallback={<div className="text-text-secondary">{text}</div>}>
            <MarkdownContent content={text || ''} />
          </Suspense>
        </div>
      )}
    </div>
  )
}
