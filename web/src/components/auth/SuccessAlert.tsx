/**
 * SuccessAlert component.
 *
 * Displays a success message in an alert box.
 *
 * @module auth/SuccessAlert
 */

import React from 'react'
import { CheckCircle } from 'lucide-react'
import { Icon } from '@/components/ui/Icon'

interface SuccessAlertProps {
  message: string
}

export const SuccessAlert = React.memo(function SuccessAlert({ message }: SuccessAlertProps) {
  return (
    <div className="bg-success/10 border border-success/30 rounded-lg p-md mb-md">
      <div className="flex items-start gap-sm">
        <Icon icon={CheckCircle} size="md" color="success" className="flex-shrink-0 mt-0.5" />
        <p className="text-sm text-success">{message}</p>
      </div>
    </div>
  )
})
