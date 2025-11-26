/**
 * RegisterPrompt component.
 *
 * Displays divider and button to switch to registration.
 *
 * @module auth/RegisterPrompt
 */

import React from 'react'

interface RegisterPromptProps {
  onSwitchToRegister: () => void
  disabled?: boolean
}

export const RegisterPrompt = React.memo(function RegisterPrompt({
  onSwitchToRegister,
  disabled,
}: RegisterPromptProps) {
  return (
    <>
      <div className="relative">
        <div className="absolute inset-0 flex items-center">
          <div className="w-full border-t border-primary/20" />
        </div>
        <div className="relative flex justify-center text-sm">
          <span className="px-sm bg-surface text-text-secondary">New to tyingshoelaces?</span>
        </div>
      </div>

      <button
        onClick={onSwitchToRegister}
        disabled={disabled}
        className="w-full text-sm text-primary hover:text-primary/80 transition-fast font-medium"
      >
        Create an account
      </button>
    </>
  )
})
