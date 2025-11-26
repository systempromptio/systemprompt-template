/**
 * Message artifacts display component.
 *
 * Shows prominent and tool execution artifacts for messages.
 *
 * @module components/chat/MessageArtifacts
 */

import React, { useMemo, useState } from 'react'
import { CompactToolDisplay } from './CompactToolDisplay'
import { StreamingArtifactRenderer } from './streaming/StreamingArtifactRenderer'
import { ChevronDown, Package } from 'lucide-react'
import { cn } from '@/lib/utils/cn'
import type { Artifact } from '@/types/artifact'
import type { ChatMessage } from '@/stores/chat.store'

interface MessageArtifactsProps {
  message: ChatMessage
  prominentArtifacts: Artifact[]
  toolExecutionArtifacts: Artifact[]
  internalArtifacts: Artifact[]
  debugMode: boolean
  isUser: boolean
  onArtifactClick: (artifactId: string) => void
}

/**
 * Memoized message artifacts component.
 * Displays artifacts in categorized sections.
 */
export const MessageArtifacts = React.memo(function MessageArtifacts({
  message,
  prominentArtifacts,
  toolExecutionArtifacts,
  internalArtifacts,
  debugMode,
  isUser,
  onArtifactClick,
}: MessageArtifactsProps) {
  const [showUsedTools, setShowUsedTools] = useState(false)

  // Filter prominent artifacts to exclude those rendered in message parts
  const uniqueProminentArtifacts = useMemo(() => {
    const artifactIdsFromParts = new Set<string>()
    message.parts?.forEach(part => {
      if (part.kind === 'data' && part.data) {
        const data = part.data as Record<string, unknown>
        if (typeof data.artifactId === 'string') {
          artifactIdsFromParts.add(data.artifactId)
        } else if (typeof data.metadata === 'object' && data.metadata) {
          const metadata = data.metadata as Record<string, unknown>
          if (typeof metadata.artifactId === 'string') {
            artifactIdsFromParts.add(metadata.artifactId)
          }
        }
      }
    })
    return prominentArtifacts.filter(a => !artifactIdsFromParts.has(a.artifactId))
  }, [message.parts, prominentArtifacts])

  if (isUser) {
    // User artifacts (all visible)
    return (
      <>
        {message.artifacts?.map((artifact: Artifact) => {
          const streamingState = message.streamingArtifacts?.get(artifact.artifactId)

          return (
            <div key={artifact.artifactId} className="mt-3 max-w-full overflow-x-auto">
              {streamingState ? (
                <StreamingArtifactRenderer
                  artifact={artifact}
                  isAppending={streamingState.isAppending}
                  isComplete={streamingState.isComplete}
                  previousParts={streamingState.previousParts}
                />
              ) : (
                <StreamingArtifactRenderer
                  artifact={artifact}
                  isComplete
                />
              )}
            </div>
          )
        })}
      </>
    )
  }

  // Assistant artifacts
  return (
    <>
      {/* Used Tools - Collapsible */}
      {(toolExecutionArtifacts.length > 0 || (debugMode && internalArtifacts.length > 0)) && (
        <details className="mb-sm" open={showUsedTools}>
          <summary
            className="cursor-pointer flex items-center gap-xs text-sm text-text-secondary hover:text-primary transition-fast font-body select-none"
            onClick={(e) => {
              e.preventDefault()
              setShowUsedTools(!showUsedTools)
            }}
          >
            <ChevronDown className={cn(
              'w-4 h-4 transition-transform',
              showUsedTools && 'rotate-180'
            )} />
            <span>
              Used {toolExecutionArtifacts.length} tool{toolExecutionArtifacts.length !== 1 ? 's' : ''}
            </span>
            {debugMode && internalArtifacts.length > 0 && (
              <span className="text-xs text-warning">
                (+{internalArtifacts.length} internal)
              </span>
            )}
          </summary>

          <div className="mt-sm space-y-xs pl-md border-l-2 border-primary-10">
            {toolExecutionArtifacts.map((artifact: Artifact) => (
              <CompactToolDisplay
                key={artifact.artifactId}
                artifact={artifact}
              />
            ))}

            {debugMode && internalArtifacts.map((artifact: Artifact) => (
              <CompactToolDisplay
                key={artifact.artifactId}
                artifact={artifact}
                dimmed
              />
            ))}
          </div>
        </details>
      )}

      {/* Prominent artifacts badge */}
      {!message.isStreaming && uniqueProminentArtifacts.length > 0 && (
        <button
          onClick={() => onArtifactClick(uniqueProminentArtifacts[0].artifactId)}
          className="mt-xs inline-flex items-center gap-xs px-md py-xs bg-primary/10 hover:bg-primary/20 border border-primary/20 rounded-lg text-sm text-primary transition-colors cursor-pointer"
        >
          <Package className="w-4 h-4" />
          <span>
            Generated {uniqueProminentArtifacts.length} artifact{uniqueProminentArtifacts.length !== 1 ? 's' : ''}
          </span>
        </button>
      )}
    </>
  )
})
