import { useChatStore } from '@/stores/chat.store'
import type { AuthRequest } from '@/stores/chat.store'

interface UseAuthRequestHandlerReturn {
  currentAuthRequest: AuthRequest | undefined
  handleAuthResponse: () => void
  handleCancelAuth: () => void
}

export function useAuthRequestHandler(): UseAuthRequestHandlerReturn {
  const {
    resolveAuthRequest,
    pendingAuthRequestsById,
    pendingAuthRequestIds,
  } = useChatStore()

  const currentAuthRequest: AuthRequest | undefined = pendingAuthRequestIds.length > 0
    ? pendingAuthRequestsById[pendingAuthRequestIds[0]]
    : undefined

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
