/**
 * ErrorAlert component.
 *
 * Displays an error message in an alert box.
 *
 * @module auth/ErrorAlert
 */

import React from 'react'
import { AlertCircle } from 'lucide-react'
import { Icon } from '@/components/ui/Icon'

interface ErrorAlertProps {
  message: string
}

export const ErrorAlert = React.memo(function ErrorAlert({ message }: ErrorAlertProps) {
  return (
    <div className="bg-error/10 border border-error/30 rounded-lg p-md mb-md">
      <div className="flex items-start gap-sm">
        <Icon icon={AlertCircle} size="md" color="error" className="flex-shrink-0 mt-0.5" />
        <p className="text-sm text-error">{message}</p>
      </div>
    </div>
  )
})
