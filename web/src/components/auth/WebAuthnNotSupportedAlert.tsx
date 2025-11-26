/**
 * WebAuthnNotSupportedAlert component.
 *
 * Displays warning when browser doesn't support WebAuthn.
 *
 * @module auth/WebAuthnNotSupportedAlert
 */

import React from 'react'
import { AlertCircle } from 'lucide-react'
import { Icon } from '@/components/ui/Icon'

export const WebAuthnNotSupportedAlert = React.memo(function WebAuthnNotSupportedAlert() {
  return (
    <div className="bg-error/10 border border-error/30 rounded-lg p-md mb-md">
      <div className="flex items-start gap-sm">
        <Icon icon={AlertCircle} size="md" color="error" className="flex-shrink-0 mt-0.5" />
        <div>
          <p className="text-sm font-semibold text-error">WebAuthn Not Supported</p>
          <p className="text-sm text-text-secondary mt-xs">
            Your browser doesn't support passkeys. Please use a modern browser like Chrome, Firefox, Safari, or Edge.
          </p>
        </div>
      </div>
    </div>
  )
})
