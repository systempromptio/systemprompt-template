/**
 * Message bubble component for chat interface.
 *
 * Displays a single message with metadata, content, artifacts,
 * and tool calls in a styled bubble.
 *
 * @module components/chat/MessageBubble
 */

import React, { useState, useMemo } from 'react'
import { cn } from '@/lib/utils/cn'
import { TaskStateIndicator } from './task/TaskStateIndicator'
import { ToolCallDisplay } from './ToolCallDisplay'
import { MessageContent } from './MessageContent'
import { MessageArtifacts } from './MessageArtifacts'
import { MessageSkills } from './MessageSkills'
import { MessageMetadata } from './MessageMetadata'
import { Avatar, Modal, ModalBody } from '@/components/ui'
import { AgentCard } from '@/components/agents/AgentCard'
import { useAuthStore } from '@/stores/auth.store'
import { useAgentStore } from '@/stores/agent.store'
import { useSettingsStore } from '@/stores/settings.store'
import { useArtifactStore } from '@/stores/artifact.store'
import { useAuth } from '@/hooks/useAuth'
import { categorizeArtifacts } from '@/lib/utils/artifact-categorization'
import type { ChatMessage } from '@/stores/chat.store'

interface MessageBubbleProps {
  message: ChatMessage
}

/**
 * Chat message bubble with metadata, content, and artifacts.
 *
 * Memoized component that composes sub-components for rendering.
 */
export const MessageBubble = React.memo(function MessageBubble({ message }: MessageBubbleProps) {
  const [showAgentModal, setShowAgentModal] = useState(false)
  const isUser = message.role === 'user'

  const username = useAuthStore((state) => state.username)
  const email = useAuthStore((state) => state.email)
  const userId = useAuthStore((state) => state.userId)
  const agents = useAgentStore((state) => state.agents)
  const debugMode = useSettingsStore((state) => state.debugMode)
  const openArtifacts = useArtifactStore((state) => state.openArtifacts)
  const { isRealUser, showLogin } = useAuth()

  const messageAgent = useMemo(
    () => agents.find(a => a.name === (message.agentId || message.task?.metadata?.agent_name)),
    [agents, message.agentId, message.task?.metadata?.agent_name]
  )

  const { internal, toolExecution, prominent } = useMemo(
    () => isUser ? { internal: [], toolExecution: [], prominent: [] } : categorizeArtifacts(message.artifacts || []),
    [isUser, message.artifacts]
  )

  return (
    <div
      className={cn(
        'flex gap-3 animate-slideInUp max-w-full',
        isUser ? 'flex-row-reverse' : 'flex-row'
      )}
    >
      {/* Avatar */}
      {isUser ? (
        <Avatar
          username={username}
          email={email}
          userId={userId}
          size="sm"
          clickable={!isRealUser}
          onClick={!isRealUser ? () => showLogin() : undefined}
        />
      ) : (
        <Avatar
          variant="agent"
          agentName={messageAgent?.name}
          agentId={messageAgent?.url}
          size="sm"
          clickable={!!messageAgent}
          onClick={() => messageAgent && setShowAgentModal(true)}
        />
      )}

      {/* Message content */}
      <div className={cn('flex-1 min-w-0', isUser && 'flex flex-col items-end')}>
        {/* Task State */}
        {!isUser && message.task && (
          <div className="mb-sm">
            <TaskStateIndicator task={message.task} compact />
          </div>
        )}

        {/* Tool Calls */}
        {!isUser && message.toolCalls && message.toolCalls.length > 0 && (
          <ToolCallDisplay
            toolCalls={message.toolCalls}
            isAgenticMode={!!message.agenticExecution}
            currentIteration={message.agenticExecution?.currentIteration}
          />
        )}

        {/* Message Content */}
        <MessageContent message={message} isUser={isUser} />

        {/* Artifacts */}
        <MessageArtifacts
          message={message}
          prominentArtifacts={prominent}
          toolExecutionArtifacts={toolExecution}
          internalArtifacts={internal}
          debugMode={debugMode}
          isUser={isUser}
          onArtifactClick={() => openArtifacts(prominent.map(a => a.artifactId))}
        />

        {/* Skills */}
        {message.contextId && (
          <>
            {console.log('[DEBUG] MessageBubble - Rendering MessageSkills:', {
              messageId: message.id,
              contextId: message.contextId,
              taskId: message.task?.id,
              hasTask: !!message.task,
              messageRole: message.role
            })}
            <MessageSkills
              taskId={message.task?.id}
              contextId={message.contextId}
            />
          </>
        )}

        {/* Metadata */}
        <MessageMetadata
          message={message}
          isUser={isUser}
        />
      </div>

      {/* Agent Modal */}
      {messageAgent && (
        <Modal
          isOpen={showAgentModal}
          onClose={() => setShowAgentModal(false)}
          title={messageAgent.name}
          size="md"
        >
          <ModalBody className="max-h-[70vh] overflow-y-auto">
            <AgentCard agent={messageAgent} />
          </ModalBody>
        </Modal>
      )}
    </div>
  )
})