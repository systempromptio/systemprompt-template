/**
 * Message content component.
 *
 * Renders message parts (text, files, data) or plain content.
 *
 * @module components/chat/MessageContent
 */

import React from 'react'
import { Card } from '@/components/ui'
import { StreamingText } from './streaming/StreamingText'
import { StreamingFile } from './streaming/StreamingFile'
import { StreamingData } from './streaming/StreamingData'
import type { Part } from '@a2a-js/sdk'
import type { ChatMessage } from '@/stores/chat.store'

interface MessageContentProps {
  message: ChatMessage
  isUser: boolean
}

/**
 * Memoized message content component.
 * Renders parts or falls back to plain content.
 */
export const MessageContent = React.memo(function MessageContent({
  message,
  isUser,
}: MessageContentProps) {
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
    <Card {...cardProps}>
      {message.parts && message.parts.length > 0 ? (
        message.parts.map((part: Part, idx: number) => (
          <div key={idx} className={idx > 0 ? 'mt-md transition-all' : 'transition-all'}>
            {part.kind === 'text' && (
              <StreamingText
                text={part.text || ''}
                isStreaming={message.isStreaming && idx === 0}
                isUser={isUser}
              />
            )}
            {part.kind === 'file' && (
              <div className="mt-md">
                <StreamingFile
                  file={part.file}
                  isComplete={!message.isStreaming}
                />
              </div>
            )}
            {part.kind === 'data' && (
              <div className="mt-md">
                <StreamingData
                  data={part.data}
                  isComplete={!message.isStreaming}
                />
              </div>
            )}
          </div>
        ))
      ) : message.content ? (
        <StreamingText
          text={message.content}
          isStreaming={message.isStreaming}
          isUser={isUser}
        />
      ) : null}
    </Card>
  )
})
