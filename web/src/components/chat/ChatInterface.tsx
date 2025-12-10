/**
 * Main chat interface component.
 *
 * Provides the interactive chat UI with message history, input field,
 * and real-time streaming support.
 *
 * Messages are displayed from the task store, which is updated via:
 * - SSE task_created events (user message appears immediately)
 * - SSE task_status_changed events (assistant response)
 * - SSE execution_step events (progress tracking)
 *
 * @module components/chat/ChatInterface
 */

import { useAgentStore } from '@/stores/agent.store'
import { useContextStore } from '@/stores/context.store'
import { useAuth } from '@/hooks/useAuth'
import { useA2AClient } from '@/hooks/useA2AClient'
import { useChatSender } from './hooks/useChatSender'
import { useTaskLoader } from './hooks/useTaskLoader'
import { useInputRequestHandler } from './hooks/useInputRequestHandler'
import { useAuthRequestHandler } from './hooks/useAuthRequestHandler'
import { MessageInput } from './MessageInput'
import { TaskList } from './TaskList'
import { SmartInputPrompt } from './input/SmartInputPrompt'
import { AuthRequiredPrompt } from './input/AuthRequiredPrompt'
import { ToolResultModal } from '@/components/tools/ToolResultModal'
import type { Message } from '@a2a-js/sdk'

export function ChatInterface() {
  const { error: clientError, retrying, retryConnection } = useA2AClient()
  const selectedAgent = useAgentStore((state) => state.selectedAgent)
  const contextError = useContextStore((state) => state.error)
  const clearContextError = useContextStore((state) => state.clearError)
  const { requireAuth, isAuthenticated } = useAuth()

  const currentContextId = useContextStore((state) => state.currentContextId)
  const { tasks, isLoading } = useTaskLoader()
  const { sendMessage: sendChatMessage, isSending, error: sendError, clearError: clearSendError } = useChatSender()
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

    // Just send the message - backend will broadcast task_created via SSE
    await sendChatMessage(text, files)
  }

  const handleInputResponseWithMessage = async (response: string) => {
    await handleInputResponse(response, handleSendMessage)
  }

  return (
    <div className="flex h-full flex-col">
      {/* Connection error banner - shows above messages instead of replacing them */}
      {clientError && (
        <div className="px-6 py-3 bg-amber-50 border-b border-amber-200">
          <div className="flex items-center gap-3">
            <div className="flex-shrink-0">
              {retrying ? (
                <svg className="animate-spin h-5 w-5 text-amber-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
              ) : (
                <svg className="w-5 h-5 text-amber-600" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                </svg>
              )}
            </div>
            <div className="flex-1">
              <p className="text-sm font-medium text-amber-800">
                {retrying ? 'Reconnecting...' : 'Connection Error'}
              </p>
              <p className="mt-0.5 text-sm text-amber-700">{clientError.message}</p>
            </div>
            {!retrying && (
              <button
                onClick={retryConnection}
                className="flex-shrink-0 px-3 py-1 text-sm font-medium text-amber-800 bg-amber-100 hover:bg-amber-200 rounded-md transition-colors"
              >
                Retry
              </button>
            )}
          </div>
        </div>
      )}

      {isLoading && (
        <div className="px-6 py-3 bg-blue-50 border-b border-blue-200">
          <div className="flex items-center gap-3">
            <div className="flex-shrink-0">
              <svg className="animate-spin h-5 w-5 text-blue-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            </div>
            <div className="text-sm text-blue-800">Loading messages...</div>
          </div>
        </div>
      )}

      {(contextError || sendError) && (
        <div className="px-6 py-3 bg-red-50 border-b border-red-200">
          <div className="flex items-start gap-3">
            <div className="flex-shrink-0 mt-0.5">
              <svg className="w-5 h-5 text-red-600" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
              </svg>
            </div>
            <div className="flex-1">
              <p className="text-sm font-medium text-red-800">Error</p>
              <p className="mt-1 text-sm text-red-700">{contextError || sendError}</p>
            </div>
            <button onClick={clearAllErrors} className="flex-shrink-0 text-red-600 hover:text-red-800">
              <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
              </svg>
            </button>
          </div>
        </div>
      )}

      <TaskList tasks={tasks} contextId={currentContextId || ''} />

      {currentInputRequest && (
        <div className="px-6">
          <SmartInputPrompt
            taskId={currentInputRequest.taskId}
            message={currentInputRequest.message as Message | undefined}
            onSubmit={handleInputResponseWithMessage}
            onCancel={handleCancelInput}
          />
        </div>
      )}

      {currentAuthRequest && (
        <div className="px-6">
          <AuthRequiredPrompt
            taskId={currentAuthRequest.taskId}
            message={currentAuthRequest.message as Message | undefined}
            onAuthorize={handleAuthResponse}
            onCancel={handleCancelAuth}
          />
        </div>
      )}

      {selectedAgent ? (
        <MessageInput
          onSend={handleSendMessage}
          disabled={isSending || !!currentInputRequest || !!currentAuthRequest}
          isStreaming={isSending}
        />
      ) : (
        <div className="border-t px-6 py-4">
          <div className="text-center text-sm text-gray-500">
            Select an agent from the sidebar to send messages
          </div>
        </div>
      )}

      <ToolResultModal />
    </div>
  )
}
