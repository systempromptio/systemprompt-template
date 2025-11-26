/**
 * AuthContextAlert component.
 *
 * Displays context about why authentication is required.
 *
 * @module auth/AuthContextAlert
 */

import React from 'react'

interface AuthContextAlertProps {
  agentName?: string
}

export const AuthContextAlert = React.memo(function AuthContextAlert({ agentName }: AuthContextAlertProps) {
  if (agentName) {
    return (
      <div className="bg-primary/10 border border-primary/30 rounded-lg p-md mb-md">
        <p className="text-sm text-text-primary">
          The agent <span className="font-semibold">{agentName}</span> requires authentication to continue.
        </p>
      </div>
    )
  }

  return (
    <div className="bg-surface-variant border border-primary/20 rounded-lg p-md mb-md">
      <p className="text-sm text-text-secondary">
        Sign in with your passkey to access authenticated features.
      </p>
    </div>
  )
})
