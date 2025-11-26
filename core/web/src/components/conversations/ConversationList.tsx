/**
 * Conversation list dropdown component.
 *
 * Displays list of conversations with rename/delete actions.
 *
 * @module components/conversations/ConversationList
 */

import React, { useCallback, useRef, useEffect } from 'react'
import { Plus, Edit2, Trash2 } from 'lucide-react'
import { Card } from '@/components/ui/Card'
import { cn } from '@/lib/utils/cn'
import { useKeyboardShortcuts } from '@/lib/accessibility'
import { ErrorBoundary } from '@/components/ErrorBoundary'
import type { Conversation } from '@/stores/context.store'

interface ConversationListProps {
  conversations: Conversation[]
  currentContextId: string | null
  renamingId: string | null
  renameName: string
  onSwitch: (id: string) => void
  onStartRename: (id: string, name: string, e: React.MouseEvent) => void
  onRename: (id: string) => void
  onDelete: (id: string, e: React.MouseEvent) => void
  onNewConversation: () => void
  onRenamingChange: (value: string) => void
  onKeyDown: (e: React.KeyboardEvent<HTMLInputElement>, id: string) => void
}

/**
 * Memoized conversation list component.
 * Displays list of conversations with actions.
 */
export const ConversationList = React.memo(function ConversationList({
  conversations,
  currentContextId,
  renamingId,
  renameName,
  onSwitch,
  onStartRename,
  onRename,
  onDelete,
  onNewConversation,
  onRenamingChange,
  onKeyDown,
}: ConversationListProps) {
  const handleClickPropagation = useCallback((e: React.MouseEvent) => {
    e.stopPropagation()
  }, [])

  const itemRefs = useRef<Map<string, HTMLDivElement>>(new Map())

  useKeyboardShortcuts(
    [
      {
        key: 'n',
        ctrl: true,
        callback: (e) => {
          e.preventDefault()
          onNewConversation()
        },
        description: 'Create new conversation',
      },
    ],
    conversations.length > 0
  )

  useEffect(() => {
    const currentIndex = conversations.findIndex((c) => c.id === currentContextId)
    if (currentIndex === -1) return

    const handleArrowKeys = (e: KeyboardEvent) => {
      if (renamingId) return

      let nextIndex = currentIndex

      if (e.key === 'ArrowDown') {
        e.preventDefault()
        nextIndex = Math.min(currentIndex + 1, conversations.length - 1)
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        nextIndex = Math.max(currentIndex - 1, 0)
      } else {
        return
      }

      if (nextIndex !== currentIndex) {
        const nextConv = conversations[nextIndex]
        onSwitch(nextConv.id)

        const element = itemRefs.current.get(nextConv.id)
        element?.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
      }
    }

    window.addEventListener('keydown', handleArrowKeys)
    return () => window.removeEventListener('keydown', handleArrowKeys)
  }, [conversations, currentContextId, onSwitch, renamingId])

  if (conversations.length === 0) {
    return (
      <Card variant="glass" padding="none" elevation="lg" className="w-full">
        <div className="px-md py-lg text-center">
          <p className="text-sm text-text-secondary font-body mb-md">
            No conversations yet
          </p>
        </div>
        <div className="border-t border-primary/10 p-sm">
          <button
            onClick={onNewConversation}
            className="w-full flex items-center justify-center gap-sm px-md py-sm text-sm text-white bg-primary hover:bg-primary/90 rounded-md transition-fast font-medium"
          >
            <Plus className="w-4 h-4" />
            <span>New Conversation</span>
          </button>
        </div>
      </Card>
    )
  }

  return (
    <ErrorBoundary fallbackVariant="compact" retryable={false}>
      <Card variant="glass" padding="none" elevation="lg" className="w-full">
        <div className="px-md py-sm border-b border-primary/10">
          <div className="text-xs font-heading font-semibold text-text-secondary uppercase tracking-wide">
            Your Conversations
          </div>
        </div>
        <div
          role="listbox"
          aria-label="Conversation list"
          aria-activedescendant={currentContextId || undefined}
          className="max-h-96 overflow-y-auto"
        >
          {conversations.map((conv) => (
            <div
              key={conv.id}
              ref={(el) => {
                if (el) {
                  itemRefs.current.set(conv.id, el)
                } else {
                  itemRefs.current.delete(conv.id)
                }
              }}
              role="option"
              aria-selected={conv.id === currentContextId}
              tabIndex={conv.id === currentContextId ? 0 : -1}
              className={cn(
                'px-sm py-xs border-b border-primary/10 last:border-b-0',
                'cursor-pointer group/item transition-fast',
                conv.id === currentContextId ? 'bg-primary/10' : 'hover:bg-primary/5',
                'focus:outline-none focus:ring-2 focus:ring-primary focus:ring-inset'
              )}
              onClick={() => onSwitch(conv.id)}
            >
            {renamingId === conv.id ? (
              <div className="px-sm py-xs" onClick={handleClickPropagation}>
                <input
                  type="text"
                  value={renameName}
                  onChange={(e) => onRenamingChange(e.target.value)}
                  onKeyDown={(e) => onKeyDown(e, conv.id)}
                  onBlur={() => onRename(conv.id)}
                  className="w-full px-sm py-xs text-sm border border-primary rounded bg-surface-dark/20 text-text-primary font-body focus:outline-none focus:ring-2 focus:ring-primary"
                  autoFocus
                />
              </div>
            ) : (
              <div className="flex items-start justify-between px-sm py-xs gap-sm">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-sm mb-xs">
                    {conv.id === currentContextId && (
                      <div className="w-1.5 h-1.5 bg-primary rounded-full flex-shrink-0" />
                    )}
                    <span className="text-sm font-heading uppercase tracking-wide text-text-primary truncate">
                      {conv.name}
                    </span>
                  </div>
                  <div className={cn(
                    'text-xs font-body font-mono',
                    conv.id === currentContextId ? 'ml-3.5' : ''
                  )}>
                    <span className="text-text-disabled">{conv.id.slice(0, 8)}</span>
                    <span className="text-text-secondary"> · {conv.messageCount} {conv.messageCount === 1 ? 'msg' : 'msgs'}</span>
                  </div>
                </div>
                <div className={cn(
                  "flex items-center gap-xs transition-fast",
                  conv.id === currentContextId
                    ? "opacity-100"
                    : "opacity-0 md:group-hover/item:opacity-100"
                )}>
                  <button
                    onClick={(e) => onStartRename(conv.id, conv.name, e)}
                    className="p-sm md:p-xs text-text-secondary hover:text-primary hover:bg-primary/10 rounded transition-fast"
                    aria-label={`Rename conversation ${conv.name}`}
                    title="Rename conversation"
                  >
                    <Edit2 className="w-3.5 h-3.5" aria-hidden="true" />
                  </button>
                  {conversations.length > 1 && (
                    <button
                      onClick={(e) => onDelete(conv.id, e)}
                      className="p-sm md:p-xs text-text-secondary hover:text-error hover:bg-error/10 rounded transition-fast"
                      aria-label={`Delete conversation ${conv.name}`}
                      title="Delete conversation"
                    >
                      <Trash2 className="w-3.5 h-3.5" aria-hidden="true" />
                    </button>
                  )}
                </div>
              </div>
            )}
          </div>
        ))}
      </div>
      <div className="border-t border-primary/10 p-sm">
        <button
          onClick={onNewConversation}
          className="w-full flex items-center justify-center gap-sm px-md py-sm text-sm text-white bg-primary hover:bg-primary/90 rounded-md transition-fast font-medium"
          aria-label="Create new conversation (Ctrl+N)"
        >
          <Plus className="w-4 h-4" />
          <span>New Conversation</span>
        </button>
      </div>
    </Card>
    </ErrorBoundary>
  )
})
