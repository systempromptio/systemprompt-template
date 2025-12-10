import { useUIStateStore } from '@/stores/ui-state.store'
import type { InputRequest } from '@/stores/ui-state.store'

interface UseInputRequestHandlerReturn {
  currentInputRequest: InputRequest | undefined
  handleInputResponse: (response: string, onSendMessage: (text: string, files?: File[]) => Promise<void>) => Promise<void>
  handleCancelInput: () => void
}

export function useInputRequestHandler(): UseInputRequestHandlerReturn {
  const resolveInputRequest = useUIStateStore((s) => s.resolveInputRequest)
  const getFirstPendingInputRequest = useUIStateStore((s) => s.getFirstPendingInputRequest)

  const currentInputRequest = getFirstPendingInputRequest()

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
