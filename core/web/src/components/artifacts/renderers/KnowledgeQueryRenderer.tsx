import type { Artifact } from '@/types/artifact'
import { Database } from 'lucide-react'

interface KnowledgeQueryRendererProps {
  artifact: Artifact
}

export function KnowledgeQueryRenderer({ artifact }: KnowledgeQueryRendererProps) {
  const dataPart = artifact.parts.find(p => p.kind === 'data')
  const data = dataPart?.kind === 'data' ? dataPart.data : null

  // Extract meaningful fields, hide internal metadata
  const content = data?.content as string | undefined
  const theme = data?.theme as string | undefined
  const found = data?.found as boolean | undefined

  if (!content && !theme) {
    return (
      <div className="p-sm bg-surface-dark/10 rounded-lg border border-primary-15">
        <div className="text-sm text-text-secondary font-body italic">
          No knowledge data available
        </div>
      </div>
    )
  }

  return (
    <div className="p-md bg-gradient-to-br from-surface to-surface-dark/30 rounded-lg border border-primary-15 transition-all">
      {/* Header */}
      <div className="flex items-center gap-sm mb-sm">
        <Database className="w-4 h-4 text-success" />
        <span className="text-xs font-heading uppercase tracking-wide text-text-secondary">
          Knowledge Base Query
        </span>
        {found !== undefined && (
          <span className={`text-xs px-xs py-0.5 rounded ${
            found
              ? 'bg-success/20 text-success border border-success/30'
              : 'bg-warning/20 text-warning border border-warning/30'
          }`}>
            {found ? 'Found' : 'Not Found'}
          </span>
        )}
      </div>

      {/* Content */}
      <div className="space-y-sm">
        {theme && (
          <div>
            <span className="text-xs font-medium text-text-secondary font-body">Theme: </span>
            <span className="text-sm text-primary font-body font-medium">
              {theme}
            </span>
          </div>
        )}

        {content && (
          <div>
            <span className="text-xs font-medium text-text-secondary font-body">Content: </span>
            <div className="mt-xs p-sm bg-surface-dark/20 rounded border border-primary-10">
              <p className="text-sm text-text-primary font-body leading-relaxed">
                {content}
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
