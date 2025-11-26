import { useState } from 'react'
import { LoginModal } from './LoginModal'
import { RegisterModal } from './RegisterModal'
import { ErrorBoundary } from '@/components/ErrorBoundary'

interface AuthModalProps {
  isOpen: boolean
  onClose: () => void
  onSuccess?: () => void
  agentName?: string
  initialMode?: 'login' | 'register'
}

export function AuthModal({ isOpen, onClose, onSuccess, agentName, initialMode = 'login' }: AuthModalProps) {
  const [mode, setMode] = useState<'login' | 'register'>(initialMode)

  return (
    <ErrorBoundary fallbackVariant="inline" showDetails={false} retryable={true}>
      {mode === 'login' ? (
        <LoginModal
          isOpen={isOpen}
          onClose={onClose}
          onSuccess={onSuccess}
          onSwitchToRegister={() => setMode('register')}
          agentName={agentName}
        />
      ) : (
        <RegisterModal
          isOpen={isOpen}
          onClose={onClose}
          onSuccess={onSuccess}
          onSwitchToLogin={() => setMode('login')}
        />
      )}
    </ErrorBoundary>
  )
}
