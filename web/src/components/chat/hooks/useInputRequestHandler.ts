import { useChatStore } from '@/stores/chat.store'
import type { InputRequest } from '@/stores/chat.store'

interface UseInputRequestHandlerReturn {
  currentInputRequest: InputRequest | undefined
  handleInputResponse: (response: string, onSendMessage: (text: string, files?: File[]) => Promise<void>) => Promise<void>
  handleCancelInput: () => void
}

export function useInputRequestHandler(): UseInputRequestHandlerReturn {
  const {
    resolveInputRequest,
    pendingInputRequestsById,
    pendingInputRequestIds,
  } = useChatStore()

  const currentInputRequest: InputRequest | undefined = pendingInputRequestIds.length > 0
    ? pendingInputRequestsById[pendingInputRequestIds[0]]
    : undefined

  const handleInputResponse = async (response: string, onSendMessage: (text: string, files?: File[]) => Promise<void>) => {
    if (!currentInputRequest) {
      throw new Error('InputRequest handler called when no request is active. This is a bug in state management.')
    }

    resolveInputRequest(currentInputRequest.taskId)
    await onSendMessage(response)
  }

  const handleCancelInput = () => {
    if (!currentInputRequest) {
      throw new Error('InputRequest cancel called when no request is active. This is a bug in state management.')
    }

    resolveInputRequest(currentInputRequest.taskId)
  }

  return {
    currentInputRequest,
    handleInputResponse,
    handleCancelInput,
  }
}
