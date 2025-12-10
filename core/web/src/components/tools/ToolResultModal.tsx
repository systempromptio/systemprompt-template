import { useEffect, useState, useMemo } from 'react'
import { Modal } from '@/components/ui/Modal'
import { ArtifactViewer } from '@/components/artifacts/ArtifactViewer'
import { useUIStateStore } from '@/stores/ui-state.store'
import { useArtifactStore } from '@/stores/artifact.store'
import { Copy, X, Info } from 'lucide-react'
import { Tab } from './Tab'
import { ExecutionMetadata } from './ExecutionMetadata'
import { LoadingState } from './LoadingState'
import { ErrorState } from './ErrorState'
import type { Artifact } from '@/types/artifact'

export function ToolResultModal() {
  const toolExecutionsById = useUIStateStore((state) => state.toolExecutionsById)
  const removeExecution = useUIStateStore((state) => state.removeToolExecution)
  const ephemeralArtifact = useUIStateStore((state) => state.ephemeralArtifact)
  const setEphemeralArtifact = useUIStateStore((state) => state.setEphemeralArtifact)
  const artifactsById = useArtifactStore((state) => state.byId)

  const executions = useMemo(() => Object.values(toolExecutionsById), [toolExecutionsById])

  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [showMetadata, setShowMetadata] = useState(false)

  const activeExecutions = useMemo(() => {
    // All executions are shown in modal by default
    return executions
  }, [executions])

  const hasModalExecutions = activeExecutions.length > 0

  useEffect(() => {
    if (hasModalExecutions && !selectedId) {
      setSelectedId(activeExecutions[0].id)
    }
  }, [activeExecutions, selectedId, hasModalExecutions])

  useEffect(() => {
    if (selectedId && !executions.find((e) => e.id === selectedId)) {
      const remainingExecution = activeExecutions[0]
      setSelectedId(remainingExecution?.id || null)
    }
  }, [executions, selectedId, activeExecutions])

  const selectedExecution = executions.find((e) => e.id === selectedId)

  const getArtifactForExecution = (artifactId?: string): Artifact | undefined => {
    if (ephemeralArtifact && ephemeralArtifact.artifactId === artifactId) {
      return ephemeralArtifact as Artifact
    }

    if (artifactId && artifactsById[artifactId]) {
      return artifactsById[artifactId]
    }

    return undefined
  }

  const selectedArtifact = selectedExecution
    ? getArtifactForExecution(selectedExecution.artifactId)
    : undefined

  const handleClose = () => {
    if (selectedExecution) {
      removeExecution(selectedExecution.id)

      if (ephemeralArtifact && ephemeralArtifact.artifactId === selectedExecution.artifactId) {
        setEphemeralArtifact(null)
      }

      if (activeExecutions.length > 1) {
        const nextExecution = activeExecutions.find((e) => e.id !== selectedId)
        setSelectedId(nextExecution?.id || null)
      }
    }
  }

  if (!hasModalExecutions || !selectedExecution) {
    return null
  }

  return (
    <Modal
      isOpen={hasModalExecutions}
      onClose={handleClose}
      size="xl"
      variant="accent"
      showCloseButton={false}
      closeOnBackdrop={true}
      closeOnEscape={true}
    >
      <div className="flex flex-col min-h-full">
        <div className="flex items-center justify-between py-3 px-4 border-b border-border">
          {activeExecutions.length > 1 ? (
            <div className="flex items-center gap-2 flex-1">
              <div className="flex gap-1 flex-1 overflow-x-auto">
                {activeExecutions.map((execution) => (
                  <Tab
                    key={execution.id}
                    label={execution.toolName}
                    isActive={selectedId === execution.id}
                    onClick={() => setSelectedId(execution.id)}
                  />
                ))}
              </div>
            </div>
          ) : (
            <h2 className="text-lg font-semibold uppercase">{selectedExecution.toolName}</h2>
          )}
          <div className="flex items-center gap-2 ml-auto">
            {selectedExecution.status === 'completed' && selectedArtifact && (
              <>
                <button
                  onClick={() => setShowMetadata(!showMetadata)}
                  className={`p-2 hover:bg-secondary/50 rounded-lg transition-all duration-200 ${showMetadata ? 'bg-secondary/30' : ''}`}
                  title="Toggle execution metadata"
                >
                  <Info className="w-4 h-4" />
                </button>
                <button
                  onClick={() => {
                    const dataStr = JSON.stringify(selectedArtifact, null, 2)
                    navigator.clipboard.writeText(dataStr)
                  }}
                  className="p-2 hover:bg-secondary/50 rounded-lg transition-colors"
                  title="Copy artifact data"
                >
                  <Copy className="w-4 h-4" />
                </button>
              </>
            )}
            <button
              onClick={handleClose}
              className="p-2 hover:bg-secondary/50 rounded-lg transition-all duration-200 hover:scale-105"
              aria-label="Close"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        <div className="flex-1 p-4">
          {selectedExecution.status === 'pending' || selectedExecution.status === 'executing' ? (
            <LoadingState toolName={selectedExecution.toolName} />
          ) : selectedExecution.status === 'error' ? (
            <ErrorState error={selectedExecution.error || 'Unknown error'} onRetry={handleClose} />
          ) : selectedArtifact ? (
            <>
              {showMetadata && <ExecutionMetadata execution={selectedExecution} />}
              <ArtifactViewer artifact={selectedArtifact} />
            </>
          ) : null}
        </div>
      </div>
    </Modal>
  )
}
