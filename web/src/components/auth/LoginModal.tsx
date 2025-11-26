import { useState, useEffect } from 'react'
import { Lock, Fingerprint } from 'lucide-react'
import { webAuthnService } from '@/services/webauthn.service'
import { useAuthStore } from '@/stores/auth.store'
import { Modal, ModalBody } from '@/components/ui/Modal'
import { Button } from '@/components/ui/Button'
import { Icon } from '@/components/ui/Icon'
import { extractUserIdFromJWT, extractEmailFromJWT } from '@/utils/jwt'
import { AuthContextAlert } from './AuthContextAlert'
import { WebAuthnNotSupportedAlert } from './WebAuthnNotSupportedAlert'
import { ErrorAlert } from './ErrorAlert'
import { SuccessAlert } from './SuccessAlert'
import { RegisterPrompt } from './RegisterPrompt'
import { ErrorBoundary } from '@/components/ErrorBoundary'

interface LoginModalProps {
  isOpen: boolean
  onClose: () => void
  onSuccess?: () => void
  onSwitchToRegister?: () => void
  agentName?: string
}

export function LoginModal({ isOpen, onClose, onSuccess, onSwitchToRegister, agentName }: LoginModalProps) {
  const [email, setEmail] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [isWebAuthnSupported, setIsWebAuthnSupported] = useState(true)

  const { setAuth } = useAuthStore()

  useEffect(() => {
    setIsWebAuthnSupported(webAuthnService.isWebAuthnSupported())
  }, [])

  useEffect(() => {
    if (!isOpen) {
      setEmail('')
      setError(null)
      setSuccess(null)
      setLoading(false)
    }
  }, [isOpen])

  const validateEmail = (email: string): boolean => {
    return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)
  }

  const handleAuthenticate = async () => {
    if (!email) {
      setError('Please enter your email address')
      return
    }

    if (!validateEmail(email)) {
      setError('Please enter a valid email address')
      return
    }

    setLoading(true)
    setError(null)
    setSuccess(null)

    const result = await webAuthnService.authenticateWithPasskey(email)

    if (result.success && result.accessToken) {
      try {
        const userId = extractUserIdFromJWT(result.accessToken)
        const userEmail = extractEmailFromJWT(result.accessToken)
        setSuccess('Authentication successful!')
        setAuth(userEmail, userId, result.accessToken, result.refreshToken || null, result.expiresIn || 3600)

        setTimeout(() => {
          onSuccess?.()
          onClose()
        }, 1000)
      } catch (jwtError) {
        setError(`Invalid authentication token: ${jwtError instanceof Error ? jwtError.message : 'Unknown error'}`)
      }
    } else {
      setError(result.error || 'Authentication failed')
    }

    setLoading(false)
  }

  return (
    <ErrorBoundary fallbackVariant="inline" showDetails={false} retryable={true}>
      <Modal
        isOpen={isOpen}
        onClose={onClose}
        variant="accent"
        size="sm"
        closeOnBackdrop={!loading}
        closeOnEscape={!loading}
      >
        <ModalBody>
          <div className="flex items-center gap-sm mb-md">
            <Icon icon={Lock} size="md" color="primary" />
            <h2 className="text-lg font-heading uppercase tracking-wide text-text-primary">
              {agentName ? 'Authentication Required' : 'Welcome Back'}
            </h2>
          </div>

          <AuthContextAlert agentName={agentName} />

          {!isWebAuthnSupported && <WebAuthnNotSupportedAlert />}

          {error && <ErrorAlert message={error} />}

          {success && <SuccessAlert message={success} />}

          <div className="space-y-md">
            <div>
              <label htmlFor="email" className="block text-sm font-medium text-text-primary mb-sm">
                Email Address
              </label>
              <input
                id="email"
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && !loading && handleAuthenticate()}
                placeholder="Enter your email address"
                className="w-full px-md py-sm border border-primary/30 rounded-lg bg-surface text-text-primary focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-fast"
                disabled={loading || !isWebAuthnSupported}
                aria-label="Email address for authentication"
                aria-invalid={error ? true : undefined}
                aria-describedby={error ? "email-error" : undefined}
              />
            </div>

            <Button
              variant="primary"
              size="md"
              icon={Fingerprint}
              iconPosition="left"
              onClick={handleAuthenticate}
              disabled={!isWebAuthnSupported}
              loading={loading}
              className="w-full"
              aria-label="Sign in with passkey"
            >
              {loading ? 'Authenticating...' : 'Sign in with Passkey'}
            </Button>

            {onSwitchToRegister && (
              <RegisterPrompt onSwitchToRegister={onSwitchToRegister} disabled={loading} />
            )}
          </div>

          <p className="text-xs text-text-secondary text-center mt-md">
            By authenticating, you agree to our{' '}
            <a href="/terms" className="text-primary hover:text-primary/80 transition-fast underline">
              terms of service
            </a>
            {' '}and{' '}
            <a href="/privacy" className="text-primary hover:text-primary/80 transition-fast underline">
              privacy policy
            </a>
            .
          </p>
        </ModalBody>
      </Modal>
    </ErrorBoundary>
  )
}
