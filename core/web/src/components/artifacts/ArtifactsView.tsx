import { useState, useMemo, useEffect } from 'react'
import { useArtifactStore } from '@/stores/artifact.store'
import { useContextStore } from '@/stores/context.store'
import { useAuthStore } from '@/stores/auth.store'
import { Card } from '@/components/ui'
import { StreamingArtifactRenderer } from '@/components/chat/streaming/StreamingArtifactRenderer'
import { formatDate } from '@/lib/utils/format'
import { ExternalLink, Package } from 'lucide-react'
import type { Artifact } from '@/types/artifact'

export function ArtifactsView() {
  const byId = useArtifactStore((state) => state.byId)
  const allIds = useArtifactStore((state) => state.allIds)
  const fetchArtifactsByContext = useArtifactStore((state) => state.fetchArtifactsByContext)
  const conversationMap = useContextStore((state) => state.conversations)
  const conversationList = useContextStore((state) => state.conversationList)
  const switchConversation = useContextStore((state) => state.switchConversation)
  const getAuthHeader = useAuthStore((state) => state.getAuthHeader)

  const artifacts = useMemo(() =>
    allIds.map(id => byId[id]).filter(Boolean),
    [allIds, byId]
  )

  const contexts = conversationList()

  const [filterType, setFilterType] = useState<string | 'all'>('all')
  const [filterContext, setFilterContext] = useState<string | 'all'>('all')

  useEffect(() => {
    const authHeader = getAuthHeader()
    if (conversationMap.size === 0 || !authHeader) {
      return
    }

    conversationMap.forEach((context) => {
      fetchArtifactsByContext(context.id, authHeader)
    })
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [conversationMap, fetchArtifactsByContext])

  // Extract unique artifact types from metadata
  const artifactTypes = useMemo(() => {
    const types = new Set<string>()
    artifacts.forEach((artifact) => {
      types.add(artifact.metadata.artifact_type)
    })
    return Array.from(types).sort()
  }, [artifacts])

  const filteredArtifacts = useMemo(() => {
    return artifacts
      .filter((artifact) => {
        const type = artifact.metadata.artifact_type
        const contextId = artifact.metadata.context_id

        if (filterType !== 'all' && type !== filterType) return false
        if (filterContext !== 'all' && contextId !== filterContext) return false
        return true
      })
      .sort((a, b) => {
        const aTime = new Date(a.metadata.created_at).getTime()
        const bTime = new Date(b.metadata.created_at).getTime()
        return bTime - aTime
      })
  }, [artifacts, filterType, filterContext])

  const handleArtifactClick = (artifact: Artifact) => {
    if ('context_id' in artifact.metadata && typeof artifact.metadata.context_id === 'string') {
      switchConversation(artifact.metadata.context_id)
    }
  }

  if (artifacts.length === 0) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center">
          <Package className="w-16 h-16 mx-auto text-text-secondary/50 mb-4" />
          <div className="text-xl font-semibold text-text-secondary">No artifacts yet</div>
          <div className="mt-2 text-sm text-text-secondary/60">
            Artifacts will appear here as agents create them
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="flex h-full flex-col">
      {/* Header with filters */}
      <div className="border-b border-border bg-background-elevated px-6 py-4">
        <div className="flex items-center justify-between gap-4">
          <h1 className="text-xl font-semibold font-display text-text-primary">
            All Artifacts
          </h1>

          <div className="flex items-center gap-3">
            {/* Type filter */}
            <select
              value={filterType}
              onChange={(e) => setFilterType(e.target.value)}
              className="px-3 py-2 text-sm rounded-lg border border-border bg-background text-text-primary font-body focus:outline-none focus:ring-2 focus:ring-primary"
            >
              <option value="all">All Types</option>
              {artifactTypes.map((type) => (
                <option key={type} value={type}>
                  {type}
                </option>
              ))}
            </select>

            {/* Context filter */}
            <select
              value={filterContext}
              onChange={(e) => setFilterContext(e.target.value)}
              className="px-3 py-2 text-sm rounded-lg border border-border bg-background text-text-primary font-body focus:outline-none focus:ring-2 focus:ring-primary"
            >
              <option value="all">All Conversations</option>
              {contexts.map((ctx) => (
                <option key={ctx.id} value={ctx.id}>
                  {ctx.name}
                </option>
              ))}
            </select>
          </div>
        </div>

        {/* Count */}
        <div className="mt-2 text-sm text-text-secondary font-body">
          Showing {filteredArtifacts.length} of {artifacts.length} artifacts
        </div>
      </div>

      {/* Artifacts gallery */}
      <div className="flex-1 overflow-y-auto p-6">
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2 xl:grid-cols-3">
          {filteredArtifacts.map((artifact) => {
            const contextId = artifact.metadata.context_id
            const artifactType = artifact.metadata.artifact_type
            const createdAt = artifact.metadata.created_at

            const context = contexts.find((c) => c.id === contextId)

            return (
              <Card
                key={artifact.artifactId}
                variant="accent"
                padding="md"
                elevation="sm"
                cutCorner="top-left"
                className="cursor-pointer transition-all hover:scale-[1.02] hover:elevation-md overflow-hidden"
                onClick={() => handleArtifactClick(artifact)}
              >
                {/* Header */}
                <div className="flex items-start justify-between gap-2 mb-3">
                  <div className="flex-1 min-w-0">
                    <div className="text-xs text-text-secondary font-body mb-1">
                      {artifact.artifactId.slice(0, 8)}
                    </div>
                    <div className="flex items-center gap-2 flex-wrap">
                      {artifactType && (
                        <div className="inline-block px-2 py-0.5 text-xs font-body rounded bg-accent/50 text-text-primary">
                          {artifactType}
                        </div>
                      )}
                      {context && (
                        <div className="inline-block px-2 py-0.5 text-xs font-body rounded bg-primary/10 text-primary">
                          {context.name}
                        </div>
                      )}
                    </div>
                  </div>
                </div>

                {/* Artifact preview */}
                <div className="mb-3 max-h-64 overflow-hidden">
                  <StreamingArtifactRenderer artifact={artifact} isComplete />
                </div>

                {/* Metadata */}
                <div className="flex items-center justify-between text-xs text-text-secondary font-body pt-3 border-t border-border">
                  <div className="flex items-center gap-2">
                    {createdAt && (
                      <span>{formatDate(new Date(createdAt))}</span>
                    )}
                  </div>
                  <div className="flex items-center gap-1 text-primary">
                    <span>Open</span>
                    <ExternalLink className="w-3 h-3" />
                  </div>
                </div>
              </Card>
            )
          })}
        </div>
      </div>
    </div>
  )
}
