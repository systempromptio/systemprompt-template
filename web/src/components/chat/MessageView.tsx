import React from 'react'
import { PartRenderer } from './PartRenderer'
import { Avatar, Card } from '@/components/ui'
import { cn } from '@/lib/utils/cn'
import { useAuthStore } from '@/stores/auth.store'
import { useAuth } from '@/hooks/useAuth'
import type { Message } from '@a2a-js/sdk'

interface MessageViewProps {
  message: Message
  agent?: { name: string; url?: string }
  isStreaming?: boolean
  onAgentClick?: () => void
  /** Optional metadata element to render inline below the bubble */
  metadata?: React.ReactNode
}

export const MessageView = React.memo(function MessageView({
  message,
  agent,
  isStreaming = false,
  onAgentClick,
  metadata,
}: MessageViewProps) {
  const isUser = message.role === 'user'

  const username = useAuthStore((s) => s.username)
  const email = useAuthStore((s) => s.email)
  const userId = useAuthStore((s) => s.userId)
  const { isRealUser, showLogin } = useAuth()

  const cardProps = isUser
    ? {
        variant: 'accent' as const,
        padding: 'md' as const,
        elevation: 'sm' as const,
        cutCorner: 'top-right' as const,
        className: 'font-body max-w-full overflow-hidden',
      }
    : {
        variant: 'accent' as const,
        padding: 'md' as const,
        elevation: 'sm' as const,
        cutCorner: 'top-left' as const,
        className: 'font-body max-w-full overflow-hidden',
      }

  return (
    <div
      className={cn(
        'flex gap-3 animate-slideInUp max-w-full mb-md',
        isUser ? 'flex-row-reverse' : 'flex-row'
      )}
    >
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
          agentName={agent?.name}
          agentId={agent?.url}
          size="sm"
          clickable={!!onAgentClick}
          onClick={onAgentClick}
        />
      )}

      <div className={cn('flex-1 min-w-0', isUser && 'flex flex-col items-end')}>
        <Card {...cardProps}>
          {message.parts && message.parts.length > 0 ? (
            message.parts.map((part, idx) => (
              <div
                key={`${message.messageId}-part-${idx}`}
                className={idx > 0 ? 'mt-md transition-all' : 'transition-all'}
              >
                <PartRenderer
                  part={part}
                  isStreaming={isStreaming && idx === 0}
                  isUser={isUser}
                />
              </div>
            ))
          ) : null}
        </Card>

        {/* Metadata rendered inline, aligned with the bubble */}
        {metadata}
      </div>
    </div>
  )
})
