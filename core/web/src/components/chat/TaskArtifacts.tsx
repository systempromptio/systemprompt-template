import React, { useState, useMemo } from 'react'
import { ChevronDown } from 'lucide-react'
import { cn } from '@/lib/utils/cn'
import { CompactToolDisplay } from './CompactToolDisplay'
import { categorizeArtifacts } from '@/lib/utils/artifact-categorization'
import { useSettingsStore } from '@/stores/settings.store'
import type { Task } from '@/types/task'
import type { Artifact } from '@/types/artifact'

interface TaskArtifactsProps {
  task: Task
  contextId: string
}

export const TaskArtifacts = React.memo(function TaskArtifacts({ task }: TaskArtifactsProps) {
  const [showUsedTools, setShowUsedTools] = useState(false)
  const debugMode = useSettingsStore((s) => s.debugMode)

  const artifacts = (task.artifacts || []) as Artifact[]

  const { internal, toolExecution } = useMemo(
    () => categorizeArtifacts(artifacts),
    [artifacts]
  )

  if (artifacts.length === 0) return null

  const hasToolSection = toolExecution.length > 0 || (debugMode && internal.length > 0)

  if (!hasToolSection) return null

  return (
    <div className="task-artifacts mt-md">
      <details className="mb-sm" open={showUsedTools}>
        <summary
          className="cursor-pointer flex items-center gap-xs text-sm text-text-secondary hover:text-primary transition-fast font-body select-none"
          onClick={(e) => {
            e.preventDefault()
            setShowUsedTools(!showUsedTools)
          }}
        >
          <ChevronDown
            className={cn('w-4 h-4 transition-transform', showUsedTools && 'rotate-180')}
          />
          <span>
            Used {toolExecution.length} tool
            {toolExecution.length !== 1 ? 's' : ''}
          </span>
          {debugMode && internal.length > 0 && (
            <span className="text-xs text-warning">(+{internal.length} internal)</span>
          )}
        </summary>

        <div className="mt-sm space-y-xs pl-md border-l-2 border-primary-10">
          {toolExecution.map((artifact) => (
            <CompactToolDisplay key={artifact.artifactId} artifact={artifact} />
          ))}

          {debugMode &&
            internal.map((artifact) => (
              <CompactToolDisplay key={artifact.artifactId} artifact={artifact} dimmed />
            ))}
        </div>
      </details>
    </div>
  )
})
