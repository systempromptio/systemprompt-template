/**
 * ExecutionMetadata component.
 *
 * Displays metadata about a tool execution including tool name, server, timing, and parameters.
 *
 * @module tools/ExecutionMetadata
 */

import React from 'react'

interface ToolExecution {
  toolName: string
  serverName: string
  executionTime?: number
  parameters?: Record<string, unknown>
}

interface ExecutionMetadataProps {
  execution: ToolExecution
}

export const ExecutionMetadata = React.memo(function ExecutionMetadata({
  execution,
}: ExecutionMetadataProps) {
  return (
    <div className="mb-4 p-3 bg-secondary/20 rounded-lg space-y-2 text-sm animate-fadeIn">
      <div className="grid grid-cols-2 gap-3">
        <div>
          <span className="text-muted">Tool:</span>
          <span className="ml-2 font-medium">{execution.toolName}</span>
        </div>
        <div>
          <span className="text-muted">Server:</span>
          <span className="ml-2 font-medium">{execution.serverName}</span>
        </div>
        {execution.executionTime && (
          <div>
            <span className="text-muted">Time:</span>
            <span className="ml-2 font-medium">{(execution.executionTime / 1000).toFixed(2)}s</span>
          </div>
        )}
      </div>
      {execution.parameters && Object.keys(execution.parameters).length > 0 && (
        <div className="mt-2 pt-2 border-t border-border">
          <span className="text-muted text-xs">Parameters:</span>
          <div className="mt-1 font-mono text-xs text-muted">
            {JSON.stringify(execution.parameters, null, 2)
              .split('\n')
              .slice(0, 4)
              .join('\n')}
            {JSON.stringify(execution.parameters, null, 2).split('\n').length > 4 && '...'}
          </div>
        </div>
      )}
    </div>
  )
})
