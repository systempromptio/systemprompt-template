/**
 * ConversationList component.
 *
 * Displays list of conversations with controls.
 *
 * @module chat/ConversationList
 */

import React from 'react'
import { Card } from '@/components/ui/Card'
import { Plus, LogOut } from 'lucide-react'
import { ConversationItem } from './ConversationItem'

interface Conversation {
  id: string
  name: string
  messageCount: number
}

interface ConversationListProps {
  conversations: Conversation[]
  currentContextId: string
  isRenaming: string | null
  renameName: string
  onSelect: (id: string) => void
  onStartRename: (id: string, currentName: string, e: React.MouseEvent) => void
  onRename: (id: string) => void
  onDelete: (id: string, e: React.MouseEvent) => void
  onRenamChange: (value: string) => void
  onNewConversation: () => void
  onLogout: () => void
  inputRef: React.RefObject<HTMLInputElement>
}

export const ConversationList = React.memo(function ConversationList({
  conversations,
  currentContextId,
  isRenaming,
  renameName,
  onSelect,
  onStartRename,
  onRename,
  onDelete,
  onRenamChange,
  onNewConversation,
  onLogout,
  inputRef,
}: ConversationListProps) {
  return (
    <Card variant="glass" padding="none" elevation="lg" className="absolute right-0 mt-sm w-72 z-modal">
      {conversations.length > 0 ? (
        <>
          <div className="px-md py-sm border-b border-primary/10">
            <div className="text-xs font-heading font-semibold text-text-secondary uppercase tracking-wide">
              Conversations
            </div>
          </div>
          <div className="max-h-64 overflow-y-auto">
            {conversations.map((conv) => (
              <ConversationItem
                key={conv.id}
                conversation={conv}
                isSelected={conv.id === currentContextId}
                isRenaming={isRenaming === conv.id}
                renameName={renameName}
                canDelete={conversations.length > 1}
                onSelect={onSelect}
                onStartRename={onStartRename}
                onRename={onRename}
                onDelete={onDelete}
                onRenamChange={onRenamChange}
                inputRef={inputRef}
              />
            ))}
          </div>
        </>
      ) : (
        <div className="px-md py-sm text-sm text-text-secondary font-body">No conversations</div>
      )}
      <div className="border-t border-primary/10 p-sm space-y-xs">
        <button
          onClick={onNewConversation}
          className="w-full flex items-center gap-sm px-sm py-sm text-sm text-primary hover:bg-primary/10 rounded-lg transition-fast font-medium"
        >
          <Plus className="w-4 h-4" />
          <span>New Conversation</span>
        </button>
        <button
          onClick={onLogout}
          className="w-full flex items-center gap-sm px-sm py-sm text-sm text-error hover:bg-error/10 rounded-lg transition-fast font-medium"
        >
          <LogOut className="w-4 h-4" />
          <span>Logout</span>
        </button>
      </div>
    </Card>
  )
})
