import { useEffect, useRef } from 'react'
import { MessageBubble } from './MessageBubble'
import { useAgentStore } from '@/stores/agent.store'
import { Avatar } from '@/components/ui/Avatar'
import { Quote, LogIn } from 'lucide-react'
import { useAuth } from '@/hooks/useAuth'
import type { ChatMessage } from '@/stores/chat.store'

interface MessageListProps {
  messages: ChatMessage[]
}

export function MessageList({ messages }: MessageListProps) {
  const bottomRef = useRef<HTMLDivElement>(null)
  const selectedAgent = useAgentStore((state) => state.selectedAgent)
  const { isRealUser, showLogin } = useAuth()

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' })
  }, [messages])

  if (messages.length === 0) {
    const agentName = selectedAgent?.name
    const agentDescription = selectedAgent?.description || 'Ready to assist you'

    return (
      <div className="flex-1 flex items-center justify-center text-text-secondary px-md py-sm md:py-xl">
        <div className="text-center max-w-2xl w-full">
          <div className="flex items-center justify-center gap-sm mb-md">
            <Avatar
              variant="agent"
              agentName={agentName}
              agentId={selectedAgent?.url}
              size="md"
              showGlow={true}
              animated={true}
            />
            <span className="text-xl font-heading font-medium uppercase tracking-wide text-primary">{agentName}</span>
          </div>
          <div className="relative mb-md px-8 md:px-0">
            <Quote className="absolute left-0 md:-left-8 top-0 w-5 h-5 md:w-6 md:h-6 text-primary/30" />
            <p className="text-sm leading-relaxed text-white font-body">
              {agentDescription}
            </p>
          </div>
          {isRealUser ? (
            <p className="text-sm">Say hello, or choose another agent from the menu</p>
          ) : (
            <button
              onClick={() => showLogin()}
              className="inline-flex items-center gap-xs px-md py-sm rounded-lg bg-primary/10 hover:bg-primary/20 border border-primary/30 hover:border-primary/60 text-primary hover:text-primary-dark transition-all duration-fast hover:scale-105"
              style={{
                borderRadius: '18px 6px 18px 18px'
              }}
            >
              <LogIn className="w-4 h-4" />
              <span className="text-sm font-body">Sign in to start chatting with {agentName}</span>
            </button>
          )}
        </div>
      </div>
    )
  }

  return (
    <div className="flex-1 overflow-y-auto overflow-x-hidden px-md py-md space-y-md scrollbar-thin max-w-full">
      {messages.map((message) => (
        <MessageBubble key={message.id} message={message} />
      ))}
      <div ref={bottomRef} />
    </div>
  )
}