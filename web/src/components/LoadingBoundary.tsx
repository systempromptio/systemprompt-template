import { Suspense, type ReactNode } from 'react'
import { Loader2 } from 'lucide-react'

interface LoadingBoundaryProps {
  children: ReactNode
  fallback?: ReactNode
  loadingVariant?: 'full' | 'inline' | 'spinner' | 'skeleton'
  loadingText?: string
  minHeight?: string
}

function DefaultLoadingFallback({
  variant = 'spinner',
  text,
  minHeight,
}: {
  variant?: 'full' | 'inline' | 'spinner' | 'skeleton'
  text?: string
  minHeight?: string
}) {
  if (variant === 'full') {
    return (
      <div
        className="min-h-screen bg-background flex items-center justify-center"
        role="status"
        aria-live="polite"
        aria-label="Loading content"
      >
        <div className="flex flex-col items-center gap-4">
          <Loader2 className="w-12 h-12 text-primary animate-spin" aria-hidden="true" />
          {text && <p className="text-text-secondary text-lg">{text}</p>}
        </div>
      </div>
    )
  }

  if (variant === 'inline') {
    return (
      <div
        className="w-full py-12 flex items-center justify-center bg-surface rounded-lg border border-border"
        style={{ minHeight: minHeight || '200px' }}
        role="status"
        aria-live="polite"
        aria-label="Loading content"
      >
        <div className="flex flex-col items-center gap-3">
          <Loader2 className="w-8 h-8 text-primary animate-spin" aria-hidden="true" />
          {text && <p className="text-text-secondary text-sm">{text}</p>}
        </div>
      </div>
    )
  }

  if (variant === 'skeleton') {
    return (
      <div
        className="w-full space-y-4 animate-pulse"
        role="status"
        aria-live="polite"
        aria-label="Loading content"
      >
        <div className="h-4 bg-surface rounded w-3/4" />
        <div className="h-4 bg-surface rounded w-full" />
        <div className="h-4 bg-surface rounded w-5/6" />
        <div className="h-4 bg-surface rounded w-2/3" />
      </div>
    )
  }

  return (
    <div className="flex items-center justify-center py-4" role="status" aria-live="polite">
      <Loader2 className="w-6 h-6 text-primary animate-spin" aria-label="Loading" />
      {text && <span className="ml-2 text-text-secondary text-sm">{text}</span>}
    </div>
  )
}

export function LoadingBoundary({
  children,
  fallback,
  loadingVariant = 'spinner',
  loadingText,
  minHeight,
}: LoadingBoundaryProps) {
  const loadingFallback =
    fallback || (
      <DefaultLoadingFallback variant={loadingVariant} text={loadingText} minHeight={minHeight} />
    )

  return <Suspense fallback={loadingFallback}>{children}</Suspense>
}

export function LoadingSpinner({
  size = 'md',
  text,
  className = '',
}: {
  size?: 'sm' | 'md' | 'lg'
  text?: string
  className?: string
}) {
  const sizeClasses = {
    sm: 'w-4 h-4',
    md: 'w-6 h-6',
    lg: 'w-8 h-8',
  }

  return (
    <div className={`flex items-center justify-center ${className}`} role="status" aria-live="polite">
      <Loader2 className={`${sizeClasses[size]} text-primary animate-spin`} aria-label="Loading" />
      {text && <span className="ml-2 text-text-secondary text-sm">{text}</span>}
    </div>
  )
}

export function LoadingOverlay({ visible, text }: { visible: boolean; text?: string }) {
  if (!visible) return null

  return (
    <div
      className="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center z-50"
      role="status"
      aria-live="polite"
      aria-label="Loading"
    >
      <div className="bg-surface border border-border rounded-lg p-6 shadow-xl">
        <div className="flex flex-col items-center gap-4">
          <Loader2 className="w-10 h-10 text-primary animate-spin" aria-hidden="true" />
          {text && <p className="text-text-primary font-medium">{text}</p>}
        </div>
      </div>
    </div>
  )
}

export function SkeletonLoader({
  lines = 4,
  className = '',
}: {
  lines?: number
  className?: string
}) {
  return (
    <div
      className={`w-full space-y-3 animate-pulse ${className}`}
      role="status"
      aria-live="polite"
      aria-label="Loading content"
    >
      {Array.from({ length: lines }, (_, i) => (
        <div
          key={i}
          className="h-4 bg-surface rounded"
          style={{ width: `${Math.random() * 30 + 70}%` }}
        />
      ))}
    </div>
  )
}

export function SkeletonCard({ className = '' }: { className?: string }) {
  return (
    <div
      className={`bg-surface border border-border rounded-lg p-6 animate-pulse ${className}`}
      role="status"
      aria-live="polite"
      aria-label="Loading"
    >
      <div className="flex items-start gap-4">
        <div className="w-12 h-12 bg-background rounded-full flex-shrink-0" />
        <div className="flex-1 space-y-3">
          <div className="h-4 bg-background rounded w-2/3" />
          <div className="h-3 bg-background rounded w-full" />
          <div className="h-3 bg-background rounded w-4/5" />
        </div>
      </div>
    </div>
  )
}

export function SkeletonTable({
  rows = 5,
  columns = 4,
  className = '',
}: {
  rows?: number
  columns?: number
  className?: string
}) {
  return (
    <div
      className={`w-full border border-border rounded-lg overflow-hidden ${className}`}
      role="status"
      aria-live="polite"
      aria-label="Loading table"
    >
      <div className="bg-surface-hover border-b border-border">
        <div className="flex gap-4 p-4">
          {Array.from({ length: columns }, (_, i) => (
            <div key={i} className="h-4 bg-surface rounded flex-1 animate-pulse" />
          ))}
        </div>
      </div>
      <div className="divide-y divide-border">
        {Array.from({ length: rows }, (_, rowIndex) => (
          <div key={rowIndex} className="flex gap-4 p-4">
            {Array.from({ length: columns }, (_, colIndex) => (
              <div key={colIndex} className="h-4 bg-surface rounded flex-1 animate-pulse" />
            ))}
          </div>
        ))}
      </div>
    </div>
  )
}

export function InlineLoader({ text, className = '' }: { text?: string; className?: string }) {
  return (
    <div className={`flex items-center gap-2 ${className}`} role="status" aria-live="polite">
      <Loader2 className="w-4 h-4 text-primary animate-spin" aria-hidden="true" />
      {text && <span className="text-text-secondary text-sm">{text}</span>}
    </div>
  )
}
