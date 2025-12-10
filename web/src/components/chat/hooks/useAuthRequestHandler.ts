import { useUIStateStore } from '@/stores/ui-state.store'
import type { AuthRequest } from '@/stores/ui-state.store'

interface UseAuthRequestHandlerReturn {
  currentAuthRequest: AuthRequest | undefined
  handleAuthResponse: () => void
  handleCancelAuth: () => void
}

export function useAuthRequestHandler(): UseAuthRequestHandlerReturn {
  const resolveAuthRequest = useUIStateStore((s) => s.resolveAuthRequest)
  const getFirstPendingAuthRequest = useUIStateStore((s) => s.getFirstPendingAuthRequest)

  const currentAuthRequest = getFirstPendingAuthRequest()

  const handleAuthResponse = () => {
    if (!currentAuthRequest) {
      throw new Error('AuthRequest handler called when no request is active. This is a bug in state management.')
    }

    resolveAuthRequest(currentAuthRequest.taskId)
  }

  const handleCancelAuth = () => {
    if (!currentAuthRequest) {
      throw new Error('AuthRequest cancel called when no request is active. This is a bug in state management.')
    }

    resolveAuthRequest(currentAuthRequest.taskId)
  }

  return {
    currentAuthRequest,
    handleAuthResponse,
    handleCancelAuth,
  }
}
