import { useEffect, useState } from 'react'
import { X, AlertCircle, ChevronDown, ChevronUp } from 'lucide-react'
import { globalErrorHandler, type ErrorInfo } from '@/lib/errorHandling'

interface ErrorToastProps {
  error: ErrorInfo
  onDismiss: () => void
}

function ErrorToastItem({ error, onDismiss }: ErrorToastProps) {
  const [expanded, setExpanded] = useState(false)
  const [autoDismiss, setAutoDismiss] = useState(true)

  useEffect(() => {
    if (!autoDismiss) return

    const timer = setTimeout(() => {
      onDismiss()
    }, 5000)

    return () => clearTimeout(timer)
  }, [autoDismiss, onDismiss])

  const handleToggleExpand = () => {
    setExpanded(!expanded)
    setAutoDismiss(false)
  }

  return (
    <div className="bg-background border-2 border-error rounded-lg shadow-lg max-w-md w-full overflow-hidden">
      <div className="p-4">
        <div className="flex items-start gap-3">
          <AlertCircle className="w-5 h-5 text-error flex-shrink-0 mt-0.5" />
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium text-error">Error</p>
            <p className="text-sm text-text-primary mt-1 break-words">
              {error.message}
            </p>
            {error.source === 'unhandledRejection' && (
              <p className="text-xs text-text-secondary mt-1">
                Unhandled promise rejection
              </p>
            )}
          </div>
          <div className="flex gap-1 flex-shrink-0">
            <button
              onClick={handleToggleExpand}
              className="p-1 hover:bg-surface rounded transition-fast text-text-secondary hover:text-text-primary"
              aria-label={expanded ? 'Collapse details' : 'Expand details'}
            >
              {expanded ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
            </button>
            <button
              onClick={onDismiss}
              className="p-1 hover:bg-surface rounded transition-fast text-text-secondary hover:text-text-primary"
              aria-label="Dismiss"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>

        {expanded && error.stack && (
          <div className="mt-3 pt-3 border-t border-border">
            <p className="text-xs font-medium text-text-secondary mb-2">Stack Trace:</p>
            <pre className="text-xs text-text-secondary font-mono bg-surface p-2 rounded overflow-auto max-h-48">
              {error.stack}
            </pre>
          </div>
        )}
      </div>

      {!expanded && (
        <div className="h-1 bg-surface relative overflow-hidden">
          <div
            className="absolute inset-0 bg-error"
            style={{
              animation: autoDismiss ? 'shrink 5s linear' : 'none',
            }}
          />
        </div>
      )}
    </div>
  )
}

export function ErrorToastContainer() {
  const [errors, setErrors] = useState<Array<ErrorInfo & { id: string }>>([])

  useEffect(() => {
    const unsubscribe = globalErrorHandler.subscribe((error) => {
      const errorWithId = {
        ...error,
        id: `${error.timestamp.getTime()}-${Math.random()}`,
      }
      setErrors((prev) => [...prev, errorWithId])
    })

    return unsubscribe
  }, [])

  const handleDismiss = (id: string) => {
    setErrors((prev) => prev.filter((e) => e.id !== id))
  }

  if (errors.length === 0) return null

  return (
    <div className="fixed top-4 right-4 z-50 flex flex-col gap-2 pointer-events-none">
      {errors.map((error) => (
        <div key={error.id} className="pointer-events-auto animate-slide-in-right">
          <ErrorToastItem
            error={error}
            onDismiss={() => handleDismiss(error.id)}
          />
        </div>
      ))}
    </div>
  )
}

const style = document.createElement('style')
style.textContent = `
  @keyframes shrink {
    from { transform: scaleX(1); }
    to { transform: scaleX(0); }
  }

  @keyframes slide-in-right {
    from {
      opacity: 0;
      transform: translateX(100%);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  .animate-slide-in-right {
    animation: slide-in-right 0.3s ease-out;
  }
`
document.head.appendChild(style)
