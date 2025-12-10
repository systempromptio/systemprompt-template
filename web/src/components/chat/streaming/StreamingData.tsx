import { useState } from 'react'
import { ChevronRight, ChevronDown } from 'lucide-react'
import { cn } from '@/lib/utils/cn'

interface StreamingDataProps {
  data: Record<string, unknown>
  isComplete?: boolean
  level?: number
}

export function StreamingData({ data, isComplete = true, level = 0 }: StreamingDataProps) {
  return (
    <div
      className={cn(
        'p-sm bg-surface-dark/10 rounded-lg border border-primary-15 transition-all font-body',
        !isComplete && 'animate-pulse'
      )}
    >
      <div className="flex items-center justify-between mb-sm">
        <span className="text-xs font-heading uppercase tracking-wide text-text-secondary">Structured Data</span>
        {!isComplete && (
          <span className="px-sm py-xs text-xs bg-primary/20 text-primary rounded font-medium">
            Updating...
          </span>
        )}
      </div>
      <JSONTree data={data} level={level} />
    </div>
  )
}

function JSONTree({ data, level = 0 }: { data: unknown; level?: number }) {
  const [expandedKeys, setExpandedKeys] = useState<Set<string>>(
    new Set(level < 2 ? Object.keys((data && typeof data === 'object' ? data : {}) as Record<string, unknown>) : [])
  )

  if (data === null) {
    return <span className="text-text-secondary italic">null</span>
  }

  if (data === undefined) {
    return <span className="text-text-secondary italic">undefined</span>
  }

  if (typeof data !== 'object') {
    return <span className={getValueColor(data)}>{JSON.stringify(data)}</span>
  }

  if (Array.isArray(data)) {
    if (data.length === 0) {
      return <span className="text-text-secondary">[]</span>
    }

    return (
      <div className="space-y-xs">
        {data.map((item, idx) => (
          <div key={idx} className="flex gap-sm">
            <span className="text-text-secondary">{idx}:</span>
            <JSONTree data={item} level={level + 1} />
          </div>
        ))}
      </div>
    )
  }

  const entries = Object.entries(data)
  if (entries.length === 0) {
    return <span className="text-text-secondary">{'{}'}</span>
  }

  const toggleKey = (key: string) => {
    setExpandedKeys((prev) => {
      const next = new Set(prev)
      if (next.has(key)) {
        next.delete(key)
      } else {
        next.add(key)
      }
      return next
    })
  }

  return (
    <div className="space-y-xs">
      {entries.map(([key, value]) => {
        const isExpandable = value && typeof value === 'object'
        const isExpanded = expandedKeys.has(key)

        return (
          <div key={key} className={cn(level > 0 && 'ml-md')}>
            <div className="flex items-start gap-sm">
              {isExpandable ? (
                <button
                  onClick={() => toggleKey(key)}
                  className="mt-0.5 hover:bg-primary/10 rounded p-xs transition-fast"
                >
                  {isExpanded ? (
                    <ChevronDown className="w-3 h-3 text-primary" />
                  ) : (
                    <ChevronRight className="w-3 h-3 text-primary" />
                  )}
                </button>
              ) : null}
              <div className="flex-1 min-w-0">
                <div className="flex items-start gap-sm">
                  <span className="font-medium text-text-primary font-body">{key}:</span>
                  {!isExpandable && <JSONTree data={value} level={level + 1} />}
                </div>
                {isExpandable && isExpanded ? (
                  <div className="mt-xs animate-fadeIn">
                    <JSONTree data={value} level={level + 1} />
                  </div>
                ) : null}
              </div>
            </div>
          </div>
        )
      })}
    </div>
  )
}

function getValueColor(value: unknown): string {
  if (typeof value === 'string') return 'text-success'
  if (typeof value === 'number') return 'text-primary'
  if (typeof value === 'boolean') return 'text-warning'
  return 'text-text-primary'
}
