/**
 * LoadingState component.
 *
 * Displays a loading state while a tool is executing.
 *
 * @module tools/LoadingState
 */

import React from 'react'
import { Loader2 } from 'lucide-react'

interface LoadingStateProps {
  toolName: string
}

export const LoadingState = React.memo(function LoadingState({ toolName }: LoadingStateProps) {
  return (
    <div className="flex flex-col items-center justify-center py-12 space-y-4">
      <Loader2 className="w-8 h-8 animate-spin text-primary" />
      <div className="text-center">
        <p className="font-medium">Executing {toolName}...</p>
        <p className="text-sm text-muted">Please wait while the tool processes</p>
      </div>
    </div>
  )
})
