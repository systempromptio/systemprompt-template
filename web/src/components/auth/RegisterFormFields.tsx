/**
 * Registration form input fields.
 *
 * Renders username, email, and full name input fields.
 *
 * @module auth/RegisterFormFields
 */

import React from 'react'
import { Fingerprint } from 'lucide-react'
import { Button } from '@/components/ui/Button'

interface RegisterFormFieldsProps {
  username: string
  onUsernameChange: (value: string) => void
  email: string
  onEmailChange: (value: string) => void
  fullName: string
  onFullNameChange: (value: string) => void
  loading: boolean
  isWebAuthnSupported: boolean
  onSubmit: () => void
}

export const RegisterFormFields = React.memo(function RegisterFormFields({
  username,
  onUsernameChange,
  email,
  onEmailChange,
  fullName,
  onFullNameChange,
  loading,
  isWebAuthnSupported,
  onSubmit,
}: RegisterFormFieldsProps) {
  return (
    <div className="space-y-md">
      <div>
        <label htmlFor="username" className="block text-sm font-medium text-text-primary mb-sm">
          Username <span className="text-error">*</span>
        </label>
        <input
          id="username"
          type="text"
          value={username}
          onChange={(e) => onUsernameChange(e.target.value)}
          placeholder="Choose a username"
          className="w-full px-md py-sm border border-primary/30 rounded-lg bg-surface text-text-primary focus:outline-none focus:ring-2 focus:ring-success focus:border-transparent transition-fast"
          disabled={loading || !isWebAuthnSupported}
          autoComplete="off"
        />
        <p className="text-xs text-text-secondary mt-xs">
          3-50 characters, letters, numbers, underscores, and hyphens only
        </p>
      </div>

      <div>
        <label htmlFor="email" className="block text-sm font-medium text-text-primary mb-sm">
          Email Address <span className="text-error">*</span>
        </label>
        <input
          id="email"
          type="email"
          value={email}
          onChange={(e) => onEmailChange(e.target.value)}
          placeholder="your.email@example.com"
          className="w-full px-md py-sm border border-primary/30 rounded-lg bg-surface text-text-primary focus:outline-none focus:ring-2 focus:ring-success focus:border-transparent transition-fast"
          disabled={loading || !isWebAuthnSupported}
          autoComplete="email"
        />
      </div>

      <div>
        <label htmlFor="fullName" className="block text-sm font-medium text-text-primary mb-sm">
          Full Name <span className="text-text-secondary">(optional)</span>
        </label>
        <input
          id="fullName"
          type="text"
          value={fullName}
          onChange={(e) => onFullNameChange(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && !loading && onSubmit()}
          placeholder="Your full name"
          className="w-full px-md py-sm border border-primary/30 rounded-lg bg-surface text-text-primary focus:outline-none focus:ring-2 focus:ring-success focus:border-transparent transition-fast"
          disabled={loading || !isWebAuthnSupported}
          autoComplete="name"
        />
      </div>

      <Button
        variant="success"
        size="md"
        icon={Fingerprint}
        iconPosition="left"
        onClick={onSubmit}
        disabled={!isWebAuthnSupported}
        loading={loading}
        className="w-full"
      >
        {loading ? 'Creating Account...' : 'Create Passkey'}
      </Button>
    </div>
  )
})
