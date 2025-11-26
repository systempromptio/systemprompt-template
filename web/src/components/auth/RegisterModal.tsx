import { useEffect } from 'react'
import { UserPlus } from 'lucide-react'
import { Modal, ModalBody } from '@/components/ui/Modal'
import { Icon } from '@/components/ui/Icon'
import { useRegisterForm } from '@/hooks/useRegisterForm'
import { WebAuthnUnsupportedAlert } from './WebAuthnUnsupportedAlert'
import { RegisterFormAlerts } from './RegisterFormAlerts'
import { RegisterFormFields } from './RegisterFormFields'
import { ErrorBoundary } from '@/components/ErrorBoundary'

interface RegisterModalProps {
  isOpen: boolean
  onClose: () => void
  onSuccess?: () => void
  onSwitchToLogin?: () => void
}

export function RegisterModal({ isOpen, onClose, onSuccess, onSwitchToLogin }: RegisterModalProps) {
  const { username, setUsername, email, setEmail, fullName, setFullName, loading, error, success, isWebAuthnSupported, reset, handleRegister } = useRegisterForm()

  useEffect(() => {
    if (!isOpen) {
      reset()
    }
  }, [isOpen, reset])

  return (
    <ErrorBoundary fallbackVariant="inline" showDetails={false} retryable={true}>
      <Modal isOpen={isOpen} onClose={onClose} variant="accent" size="sm" closeOnBackdrop={!loading} closeOnEscape={!loading}>
        <ModalBody>
          <div className="flex items-center gap-sm mb-md">
            <Icon icon={UserPlus} size="md" color="success" />
            <h2 className="text-lg font-heading uppercase tracking-wide text-text-primary">Create Your Account</h2>
          </div>

          <div className="bg-surface-variant border border-success/20 rounded-lg p-md mb-md">
            <p className="text-sm text-text-secondary">Register with a passkey for secure, passwordless authentication.</p>
          </div>

          <WebAuthnUnsupportedAlert isVisible={!isWebAuthnSupported} />
          <RegisterFormAlerts error={error} success={success} />

          <RegisterFormFields
            username={username}
            onUsernameChange={setUsername}
            email={email}
            onEmailChange={setEmail}
            fullName={fullName}
            onFullNameChange={setFullName}
            loading={loading}
            isWebAuthnSupported={isWebAuthnSupported}
            onSubmit={() => handleRegister(onSuccess)}
          />

          {onSwitchToLogin && (
            <>
              <div className="relative mt-md">
                <div className="absolute inset-0 flex items-center">
                  <div className="w-full border-t border-primary/20" />
                </div>
                <div className="relative flex justify-center text-sm">
                  <span className="px-sm bg-surface text-text-secondary">Already have an account?</span>
                </div>
              </div>

              <button
                onClick={onSwitchToLogin}
                disabled={loading}
                className="w-full text-sm text-primary hover:text-primary/80 transition-fast font-medium mt-md"
                aria-label="Sign in to existing account"
              >
                Sign in instead
              </button>
            </>
          )}

          <p className="text-xs text-text-secondary text-center mt-md">
            By creating an account, you agree to our{' '}
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
