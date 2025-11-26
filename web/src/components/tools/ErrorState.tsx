/**
 * ErrorState component.
 *
 * Displays an error state when a tool execution fails.
 *
 * @module tools/ErrorState
 */

import React from 'react'
import { AlertCircle } from 'lucide-react'
import { Button } from '@/components/ui/Button'

interface ErrorStateProps {
  error: string
  onRetry: () => void
}

export const ErrorState = React.memo(function ErrorState({ error, onRetry }: ErrorStateProps) {
  return (
    <div className="flex flex-col items-center justify-center py-12 space-y-4">
      <AlertCircle className="w-8 h-8 text-error" />
      <div className="text-center max-w-md">
        <p className="font-medium text-error mb-2">Execution Failed</p>
        <p className="text-sm text-muted mb-4">{error}</p>
        <Button onClick={onRetry} variant="secondary" size="sm">
          Retry
        </Button>
      </div>
    </div>
  )
})
