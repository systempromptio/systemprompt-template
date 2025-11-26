import type { FC } from 'react'
import { MessageInput } from './MessageInput'
import { MessageList } from './MessageList'
import { SmartInputPrompt } from './input/SmartInputPrompt'
import { AuthRequiredPrompt } from './input/AuthRequiredPrompt'
import { ToolResultModal } from '@/components/tools/ToolResultModal'
import { ArtifactModal } from '@/components/artifacts/ArtifactModal'
import type { ChatMessage } from '@/stores/chat.store'

interface ChatContentProps {
  messages: ChatMessage[]
  isLoading: boolean
  contextError: string | null
  sendError: string | null
  isSending: boolean
  selectedAgent: any | null
  currentInputRequest: any | undefined
  currentAuthRequest: any | undefined
  onSendMessage: (text: string, files?: File[]) => Promise<void>
  onInputResponse: (response: string) => Promise<void>
  onCancelInput: () => void
  onAuthResponse: () => void
  onCancelAuth: () => void
  onClearErrors: () => void
}

export const ChatContent: FC<ChatContentProps> = ({
  messages,
  isLoading,
  contextError,
  sendError,
  isSending,
  selectedAgent,
  currentInputRequest,
  currentAuthRequest,
  onSendMessage,
  onInputResponse,
  onCancelInput,
  onAuthResponse,
  onCancelAuth,
  onClearErrors,
}) => {
  return (
    <div className="flex h-full flex-col">
      {/* Loading Indicator */}
      {isLoading && (
        <div className="px-6 py-3 bg-blue-50 border-b border-blue-200">
          <div className="flex items-center gap-3">
            <div className="flex-shrink-0">
              <svg className="animate-spin h-5 w-5 text-blue-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            </div>
            <div className="text-sm text-blue-800">
              Loading messages...
            </div>
          </div>
        </div>
      )}

      {/* Error Notification */}
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
            <button
              onClick={onClearErrors}
              className="flex-shrink-0 text-red-600 hover:text-red-800"
            >
              <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
              </svg>
            </button>
          </div>
        </div>
      )}

      {/* Messages */}
      <MessageList messages={messages} />

      {/* Smart Input Prompts */}
      {currentInputRequest && (
        <div className="px-6">
          <SmartInputPrompt
            taskId={currentInputRequest.taskId}
            message={currentInputRequest.message}
            onSubmit={onInputResponse}
            onCancel={onCancelInput}
          />
        </div>
      )}

      {currentAuthRequest && (
        <div className="px-6">
          <AuthRequiredPrompt
            taskId={currentAuthRequest.taskId}
            message={currentAuthRequest.message}
            onAuthorize={onAuthResponse}
            onCancel={onCancelAuth}
          />
        </div>
      )}

      {/* Input */}
      {selectedAgent ? (
        <MessageInput
          onSend={onSendMessage}
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

      {/* Tool Result Modal */}
      <ToolResultModal />

      {/* Artifact Modal */}
      <ArtifactModal />
    </div>
  )
}
