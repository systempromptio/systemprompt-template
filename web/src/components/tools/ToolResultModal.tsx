import { useEffect, useState } from 'react'
import { Modal } from '@/components/ui/Modal'
import { ArtifactViewer } from '@/components/artifacts/ArtifactViewer'
import { useToolExecutionStore } from '@/stores/toolExecution.store'
import { Copy, X, Info } from 'lucide-react'
import { Tab } from './Tab'
import { ExecutionMetadata } from './ExecutionMetadata'
import { LoadingState } from './LoadingState'
import { ErrorState } from './ErrorState'

export function ToolResultModal() {
  const executions = useToolExecutionStore((state) => state.executions)
  const removeExecution = useToolExecutionStore((state) => state.removeExecution)
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [showMetadata, setShowMetadata] = useState(false)

  const shouldShow = executions.length > 0
  const activeExecutions = executions.filter(
    (e) => e.renderBehavior === 'modal' || e.renderBehavior === 'both'
  )
  const hasModalExecutions = activeExecutions.length > 0

  useEffect(() => {
    if (hasModalExecutions && !selectedId) {
      setSelectedId(activeExecutions[0].id)
    }
  }, [activeExecutions, selectedId])

  useEffect(() => {
    if (selectedId && !executions.find((e) => e.id === selectedId)) {
      const remainingExecution = activeExecutions[0]
      setSelectedId(remainingExecution?.id || null)
    }
  }, [executions, selectedId, activeExecutions])

  const selectedExecution = executions.find((e) => e.id === selectedId)

  const handleClose = () => {
    if (selectedExecution) {
      removeExecution(selectedExecution.id)
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
      isOpen={shouldShow && hasModalExecutions}
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
            {selectedExecution.status === 'completed' && selectedExecution.artifact && (
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
                    const dataStr = JSON.stringify(selectedExecution.artifact, null, 2)
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
          ) : selectedExecution.artifact ? (
            <>
              {showMetadata && <ExecutionMetadata execution={selectedExecution} />}
              <ArtifactViewer artifact={selectedExecution.artifact} />
            </>
          ) : null}
        </div>
      </div>
    </Modal>
  )
}
