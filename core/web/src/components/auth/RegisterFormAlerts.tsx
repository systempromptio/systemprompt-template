/**
 * Registration form error and success alerts.
 *
 * Displays error or success messages for registration.
 *
 * @module auth/RegisterFormAlerts
 */

import React from 'react'
import { CheckCircle, AlertCircle } from 'lucide-react'
import { Icon } from '@/components/ui/Icon'

interface RegisterFormAlertsProps {
  error: string | null
  success: string | null
}

export const RegisterFormAlerts = React.memo(function RegisterFormAlerts({ error, success }: RegisterFormAlertsProps) {
  return (
    <>
      {error && (
        <div className="bg-error/10 border border-error/30 rounded-lg p-md mb-md">
          <div className="flex items-start gap-sm">
            <Icon icon={AlertCircle} size="md" color="error" className="flex-shrink-0 mt-0.5" />
            <p className="text-sm text-error">{error}</p>
          </div>
        </div>
      )}

      {success && (
        <div className="bg-success/10 border border-success/30 rounded-lg p-md mb-md">
          <div className="flex items-start gap-sm">
            <Icon icon={CheckCircle} size="md" color="success" className="flex-shrink-0 mt-0.5" />
            <p className="text-sm text-success">{success}</p>
          </div>
        </div>
      )}
    </>
  )
})
