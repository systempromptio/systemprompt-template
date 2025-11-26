/**
 * Artifact viewer header component.
 *
 * Displays artifact title, type badge, and action buttons.
 *
 * @module artifacts/viewer/ArtifactHeader
 */

import React from 'react'
import { Code2, RefreshCw, X, ChevronLeft, ChevronRight } from 'lucide-react'
import type { Artifact } from '@/types/artifact'
import { ArtifactIcon } from './ArtifactIcon'

interface ArtifactHeaderProps {
  artifact: Artifact
  artifactType: string
  isMcpTool: boolean
  isRefreshing: boolean
  isEphemeral: boolean
  onShowRawData: () => void
  onRefresh: () => void
  onClose?: () => void
  onPrevious?: () => void
  onNext?: () => void
  currentIndex?: number
  totalCount?: number
  metadata: { artifact_type?: string } | null
}

function toTitleCase(str: string): string {
  return str
    .replace(/[_-]/g, ' ')
    .split(' ')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
    .join(' ')
}

export const ArtifactHeader = React.memo(function ArtifactHeader({
  artifact,
  artifactType,
  isMcpTool,
  isRefreshing,
  isEphemeral,
  onShowRawData,
  onRefresh,
  onClose,
  onPrevious,
  onNext,
  currentIndex,
  totalCount,
  metadata,
}: ArtifactHeaderProps) {
  const hasMultiple = totalCount && totalCount > 1
  return (
    <div className="px-6 py-4 border-b border-primary-10 bg-surface-variant/50 flex items-center justify-between flex-wrap gap-2">
      <div className="flex items-center gap-2">
        <ArtifactIcon artifactType={artifactType} />
        <h4 className="uppercase px-2 py-1 text-primary font-medium">{toTitleCase(artifact.name || 'Artifact')}</h4>
        {metadata?.artifact_type && (
          <span className="text-xs px-2 py-0.5 bg-primary/20 text-primary border border-primary-15 rounded">
            {toTitleCase(metadata.artifact_type)}
          </span>
        )}
        {hasMultiple && (
          <span className="text-xs px-2 py-0.5 text-secondary">
            {(currentIndex || 0) + 1} of {totalCount}
          </span>
        )}
      </div>

      <div className="flex items-center gap-2">
        {isMcpTool && (
          <>
            {artifact.description && (
              <span className="text-sm text-secondary mr-2">{artifact.description}</span>
            )}

            <button
              onClick={onShowRawData}
              className="p-2 hover:bg-surface-dark rounded-lg transition-all duration-200"
              title="View raw structured data"
            >
              <Code2 className="w-4 h-4 text-primary" />
            </button>

            {!isEphemeral && (
              <button
                onClick={onRefresh}
                disabled={isRefreshing}
                className="p-2 hover:bg-surface-dark rounded-lg transition-all duration-200 disabled:opacity-50"
                title="Refresh data"
              >
                <RefreshCw className={`w-4 h-4 text-primary ${isRefreshing ? 'animate-spin' : ''}`} />
              </button>
            )}
          </>
        )}

        {!isMcpTool && artifact.description && (
          <span className="text-sm text-secondary mr-2">{artifact.description}</span>
        )}

        {hasMultiple && (
          <>
            <button
              onClick={onPrevious}
              className="p-2 hover:bg-surface-dark rounded-lg transition-all duration-200"
              title="Previous artifact"
              aria-label="Previous artifact"
            >
              <ChevronLeft className="w-4 h-4 text-primary" />
            </button>

            <button
              onClick={onNext}
              className="p-2 hover:bg-surface-dark rounded-lg transition-all duration-200"
              title="Next artifact"
              aria-label="Next artifact"
            >
              <ChevronRight className="w-4 h-4 text-primary" />
            </button>
          </>
        )}

        {onClose && (
          <button
            onClick={onClose}
            className="p-2 hover:bg-surface-dark rounded-lg transition-all duration-200"
            title="Close"
            aria-label="Close"
          >
            <X className="w-4 h-4 text-primary" />
          </button>
        )}
      </div>
    </div>
  )
})
