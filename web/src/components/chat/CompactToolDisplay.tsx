import { useState } from 'react'
import type { Artifact } from '@a2a-js/sdk'
import { ChevronDown, Wrench, AlertCircle } from 'lucide-react'
import { cn } from '@/lib/utils/cn'

interface CompactToolDisplayProps {
  artifact: Artifact
  dimmed?: boolean
}

export function CompactToolDisplay({ artifact, dimmed = false }: CompactToolDisplayProps) {
  const [expanded, setExpanded] = useState(false)
  const toolName = artifact.metadata?.tool_name as string | undefined
  const isError = artifact.metadata?.type === 'tool_error'
  const data = artifact.parts.find(p => p.kind === 'data')?.kind === 'data'
    ? artifact.parts.find(p => p.kind === 'data')?.data
    : null

  return (
    <div
      className={cn(
        'rounded-lg border transition-all',
        dimmed ? 'border-warning/30 bg-warning/5 opacity-60' : 'border-primary-10 bg-surface',
        isError && 'border-error/30 bg-error/5'
      )}
    >
      {/* Header - Always visible */}
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center justify-between p-sm hover:bg-surface-variant transition-fast"
      >
        <div className="flex items-center gap-sm">
          {isError ? (
            <AlertCircle className="w-4 h-4 text-error flex-shrink-0" />
          ) : (
            <Wrench className="w-4 h-4 text-primary flex-shrink-0" />
          )}
          <span className={cn(
            'text-sm font-medium font-body',
            isError ? 'text-error' : 'text-text-primary'
          )}>
            {toolName || 'Unknown tool'}
          </span>
          {dimmed && (
            <span className="text-xs px-xs py-0.5 bg-warning/20 text-warning border border-warning/30 rounded">
              internal
            </span>
          )}
          {isError && (
            <span className="text-xs px-xs py-0.5 bg-error/20 text-error border border-error/30 rounded">
              error
            </span>
          )}
        </div>
        <ChevronDown
          className={cn(
            'w-4 h-4 text-text-secondary transition-transform',
            expanded && 'rotate-180'
          )}
        />
      </button>

      {/* Expandable content */}
      {expanded && (
        <div className="px-sm pb-sm border-t border-primary-10">
          <div className="mt-sm">
            {artifact.description && (
              <p className="text-xs text-text-secondary mb-sm font-body">
                {artifact.description}
              </p>
            )}

            {data && (
              <div className="bg-surface-dark/50 rounded p-xs overflow-x-auto">
                <pre className="text-xs text-text-primary font-mono">
                  {JSON.stringify(data, null, 2)}
                </pre>
              </div>
            )}

            {!data && (
              <div className="text-xs text-text-secondary italic font-body">
                No data available
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
