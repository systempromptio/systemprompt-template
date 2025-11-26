/**
 * ConversationItem component.
 *
 * Displays a single conversation item with rename/delete actions.
 *
 * @module chat/ConversationItem
 */

import React, { useEffect } from 'react'
import { Edit2, Trash2 } from 'lucide-react'

interface Conversation {
  id: string
  name: string
  messageCount: number
}

interface ConversationItemProps {
  conversation: Conversation
  isSelected: boolean
  isRenaming: boolean
  renameName: string
  canDelete: boolean
  onSelect: (id: string) => void
  onStartRename: (id: string, currentName: string, e: React.MouseEvent) => void
  onRename: (id: string) => void
  onDelete: (id: string, e: React.MouseEvent) => void
  onRenamChange: (value: string) => void
  inputRef: React.RefObject<HTMLInputElement>
}

export const ConversationItem = React.memo(function ConversationItem({
  conversation,
  isSelected,
  isRenaming,
  renameName,
  canDelete,
  onSelect,
  onStartRename,
  onRename,
  onDelete,
  onRenamChange,
  inputRef,
}: ConversationItemProps) {
  useEffect(() => {
    if (isRenaming && inputRef.current) {
      inputRef.current.focus()
      inputRef.current.select()
    }
  }, [isRenaming, inputRef])

  return (
    <div
      className={`px-sm py-xs border-b border-primary/10 last:border-b-0 ${
        isSelected ? 'bg-primary/10' : 'hover:bg-primary/5'
      } cursor-pointer group transition-fast`}
      onClick={() => onSelect(conversation.id)}
    >
      {isRenaming ? (
        <div className="px-sm py-xs" onClick={(e) => e.stopPropagation()}>
          <input
            ref={inputRef}
            type="text"
            value={renameName}
            onChange={(e) => onRenamChange(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') onRename(conversation.id)
              if (e.key === 'Escape') onRename(conversation.id)
            }}
            onBlur={() => onRename(conversation.id)}
            className="w-full px-sm py-xs text-sm border border-primary rounded bg-surface-dark/20 text-text-primary font-body focus:outline-none focus:ring-2 focus:ring-primary"
          />
        </div>
      ) : (
        <div className="flex items-center justify-between px-sm py-xs">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-sm">
              {isSelected && <div className="w-1.5 h-1.5 bg-primary rounded-full" />}
              <span className="text-sm font-heading uppercase tracking-wide text-text-primary truncate">
                {conversation.name}
              </span>
            </div>
            <div className="text-xs text-text-secondary ml-3.5 font-body">{conversation.messageCount} messages</div>
          </div>
          <div className="flex items-center gap-xs opacity-0 group-hover:opacity-100 transition-fast">
            <button
              onClick={(e) => onStartRename(conversation.id, conversation.name, e)}
              className="p-xs text-text-secondary hover:text-primary hover:bg-primary/10 rounded transition-fast"
              title="Rename"
            >
              <Edit2 className="w-3.5 h-3.5" />
            </button>
            {canDelete && (
              <button
                onClick={(e) => onDelete(conversation.id, e)}
                className="p-xs text-text-secondary hover:text-error hover:bg-error/10 rounded transition-fast"
                title="Delete"
              >
                <Trash2 className="w-3.5 h-3.5" />
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  )
})
