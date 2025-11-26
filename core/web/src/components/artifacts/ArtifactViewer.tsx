import { useState, useCallback, useEffect } from 'react'
import type { Artifact } from '@/types/artifact'
import { isEphemeralArtifact } from '@/types/artifact'
import type { ToolExecutionResponse } from '@/types/mcp'
import { RawDataModal } from './RawDataModal'
import { ArtifactHeader } from './viewer/ArtifactHeader'
import { ArtifactContentRenderer } from './viewer/ArtifactContentRenderer'
import { extractMetadata } from '@/lib/artifacts'
import { useMcpToolCaller } from '@/hooks/useMcpToolCaller'
import { useArtifactSubscription } from '@/hooks/useArtifactSubscription'
import { useAuthStore } from '@/stores/auth.store'
import { useArtifactStore } from '@/stores/artifact.store'
import { getApiBaseUrl } from '@/utils/env'

interface ArtifactViewerProps {
  artifact: Artifact
  onClose?: () => void
}

export function ArtifactViewer({ artifact: initialArtifact, onClose }: ArtifactViewerProps) {
  const [artifact, setArtifact] = useState(initialArtifact)
  const [showRawDataModal, setShowRawDataModal] = useState(false)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const { callTool } = useMcpToolCaller()
  const authHeader = useAuthStore(state => state.getAuthHeader())
  const nextArtifact = useArtifactStore(state => state.nextArtifact)
  const previousArtifact = useArtifactStore(state => state.previousArtifact)
  const selectedArtifactIds = useArtifactStore(state => state.selectedArtifactIds)
  const currentArtifactIndex = useArtifactStore(state => state.currentArtifactIndex)

  useEffect(() => {
    setArtifact(initialArtifact)
  }, [initialArtifact])

  const handleArtifact = useCallback((newArtifact: Artifact) => {
    setArtifact(newArtifact)
    setIsRefreshing(false)
  }, [])

  const { subscribe } = useArtifactSubscription({
    onArtifact: handleArtifact,
    onTimeout: () => setIsRefreshing(false),
    timeout: 30000
  })

  const metadata = extractMetadata(artifact)
  const artifactType = metadata?.artifact_type || 'json'
  const toolName = metadata?.tool_name as string | undefined
  const isMcpTool = metadata?.source === 'mcp_tool'
  const artifactIsEphemeral = isEphemeralArtifact(artifact)

  const handleRefresh = useCallback(async () => {
    const executionId = metadata?.mcp_execution_id as string | undefined
    if (!executionId) return

    setIsRefreshing(true)
    try {
      const headers: HeadersInit = { 'Content-Type': 'application/json' }
      if (authHeader) headers['Authorization'] = authHeader

      const response = await fetch(`/api/v1/mcp/executions/${executionId}`, { headers })
      if (!response.ok) throw new Error(`Failed to fetch execution: ${response.statusText}`)

      const execution: ToolExecutionResponse = await response.json()
      const newExecutionId = crypto.randomUUID()
      subscribe(newExecutionId)

      const serverEndpoint = `${getApiBaseUrl()}${execution.server_endpoint}`
      await callTool(serverEndpoint, execution.tool_name, execution.input)
    } catch {
      setIsRefreshing(false)
    }
  }, [metadata, authHeader, callTool, subscribe])

  return (
    <>
      <div className="h-full max-w-full flex flex-col">
        <ArtifactHeader
          artifact={artifact}
          artifactType={artifactType}
          isMcpTool={isMcpTool}
          isRefreshing={isRefreshing}
          isEphemeral={artifactIsEphemeral}
          onShowRawData={() => setShowRawDataModal(true)}
          onRefresh={handleRefresh}
          onClose={onClose}
          onPrevious={previousArtifact}
          onNext={nextArtifact}
          currentIndex={currentArtifactIndex}
          totalCount={selectedArtifactIds.length}
          metadata={metadata}
        />
        <div className="flex-1 overflow-auto">
          <ArtifactContentRenderer
            artifact={artifact}
            artifactType={artifactType}
            toolName={toolName}
            metadata={metadata}
          />
        </div>
      </div>
      {showRawDataModal && (
        <RawDataModal artifact={artifact} isOpen={showRawDataModal} onClose={() => setShowRawDataModal(false)} />
      )}
    </>
  )
}