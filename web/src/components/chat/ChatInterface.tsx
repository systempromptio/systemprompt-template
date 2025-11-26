/**
 * Main chat interface component.
 *
 * Provides the interactive chat UI with message history, input field,
 * and real-time streaming support. Composed of smaller sub-components
 * for better maintainability.
 *
 * @module components/chat/ChatInterface
 */

import { useAgentStore } from '@/stores/agent.store'
import { useContextStore } from '@/stores/context.store'
import { useAuth } from '@/hooks/useAuth'
import { useA2AClient } from '@/hooks/useA2AClient'
import { useChatSender } from './hooks/useChatSender'
import { useChatMessageLoader } from './hooks/useChatMessageLoader'
import { useInputRequestHandler } from './hooks/useInputRequestHandler'
import { useAuthRequestHandler } from './hooks/useAuthRequestHandler'
import { ConnectionError } from './ConnectionError'
import { ChatContent } from './ChatContent'
import type { ChatMessage } from '@/stores/chat.store'

/**
 * Chat interface component.
 *
 * Displays conversation history and provides message input with
 * support for both regular and streaming messages.
 *
 * Features:
 * - Real-time message streaming
 * - Artifact accumulation and display
 * - Tool execution results
 * - Optimistic UI updates
 *
 * @example
 * ```typescript
 * function App() {
 *   return (
 *     <AppLayout>
 *       <ChatInterface />
 *     </AppLayout>
 *   )
 * }
 * ```
 */
export function ChatInterface() {
  const { error: clientError, retrying, retryConnection } = useA2AClient()
  const selectedAgent = useAgentStore((state) => state.selectedAgent)
  const currentContextId = useContextStore((state) => state.currentContextId)
  const contextError = useContextStore((state) => state.error)
  const clearContextError = useContextStore((state) => state.clearError)
  const { requireAuth, isAuthenticated } = useAuth()

  const { messages, isLoading, setOptimisticMessages } = useChatMessageLoader()
  const { sendMessage: sendChatMessage, isSending, error: sendError, clearError: clearSendError } = useChatSender({ setOptimisticMessages })
  const { currentInputRequest, handleInputResponse, handleCancelInput } = useInputRequestHandler()
  const { currentAuthRequest, handleAuthResponse, handleCancelAuth } = useAuthRequestHandler()

  const clearAllErrors = () => {
    clearContextError()
    clearSendError()
  }

  const handleSendMessage = async (text: string, files?: File[]) => {
    if (isSending || !selectedAgent) return

    const agentRequiresAuth = selectedAgent.security && selectedAgent.security.length > 0

    if (agentRequiresAuth && !isAuthenticated) {
      requireAuth(selectedAgent?.name, () => {
        handleSendMessage(text, files)
      })
      return
    }

    // Add optimistic user message for immediate feedback
    const userMessage: ChatMessage = {
      id: crypto.randomUUID(),
      timestamp: new Date(),
      role: 'user',
      content: text,
      parts: [{ kind: 'text', text }],
      contextId: currentContextId,
      agentId: selectedAgent.name,
      isStreaming: false, // User message is complete immediately
    }

    // Add optimistic assistant message to show loading state
    // Use temp- prefix so we can identify and replace it with real messageId from stream
    const tempAssistantId = `temp-${crypto.randomUUID()}`
    const assistantMessage: ChatMessage = {
      id: tempAssistantId,
      timestamp: new Date(),
      role: 'assistant',
      content: '',
      parts: [{ kind: 'text', text: '' }],
      contextId: currentContextId,
      agentId: selectedAgent.name,
      isStreaming: true,
    }

    setOptimisticMessages((prev: ChatMessage[]) => [...prev, userMessage, assistantMessage])
    await sendChatMessage(text, files, tempAssistantId)
  }

  const handleInputResponseWithMessage = async (response: string) => {
    await handleInputResponse(response, handleSendMessage)
  }

  if (clientError) {
    return <ConnectionError error={clientError} isRetrying={retrying} onRetry={retryConnection} />
  }

  return (
    <ChatContent
      messages={messages}
      isLoading={isLoading}
      contextError={contextError}
      sendError={sendError}
      isSending={isSending}
      selectedAgent={selectedAgent}
      currentInputRequest={currentInputRequest}
      currentAuthRequest={currentAuthRequest}
      onSendMessage={handleSendMessage}
      onInputResponse={handleInputResponseWithMessage}
      onCancelInput={handleCancelInput}
      onAuthResponse={handleAuthResponse}
      onCancelAuth={handleCancelAuth}
      onClearErrors={clearAllErrors}
    />
  )
}
