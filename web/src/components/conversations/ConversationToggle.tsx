/**
 * Conversation toggle component.
 *
 * Dropdown button for managing conversations.
 *
 * @module components/conversations/ConversationToggle
 */

import React, { useState, useRef, useCallback } from 'react'
import { useContextStore } from '@/stores/context.store'
import { useAuth } from '@/hooks/useAuth'
import { useClickOutside } from '@/hooks/useClickOutside'
import { ConversationToggleButton } from './ConversationToggleButton'
import { ConversationList } from './ConversationList'

interface ConversationToggleProps {
  isSelected?: boolean
  disabled?: boolean
  onViewChange?: () => void
}

/**
 * Memoized conversation toggle component.
 * Manages dropdown state and handles conversation operations.
 */
export const ConversationToggle = React.memo(function ConversationToggle({
  isSelected = false,
  disabled = false,
  onViewChange,
}: ConversationToggleProps) {
  const [isOpen, setIsOpen] = useState(false)
  const [renamingId, setRenamingId] = useState<string | null>(null)
  const [renameName, setRenameName] = useState('')
  const dropdownRef = useRef<HTMLDivElement>(null)

  const {
    conversationList,
    currentContextId,
    getCurrentConversation,
    createConversation,
    switchConversation,
    renameConversation,
    deleteConversation,
  } = useContextStore()

  const { isAuthenticated, showLogin } = useAuth()
  const conversations = conversationList()
  const currentConversation = getCurrentConversation()

  // Set up click outside detection
  useClickOutside(dropdownRef, useCallback(() => {
    setIsOpen(false)
    setRenamingId(null)
  }, []))

  // Event handlers
  const handleToggleClick = useCallback(() => {
    if (!isAuthenticated) {
      showLogin()
      return
    }
    setIsOpen(!isOpen)
    if (onViewChange && !isSelected) {
      onViewChange()
    }
  }, [isAuthenticated, isOpen, isSelected, showLogin, onViewChange])

  const handleNewConversation = useCallback(() => {
    createConversation()
    setIsOpen(false)
  }, [createConversation])

  const handleSwitch = useCallback((id: string) => {
    if (id !== currentContextId) {
      switchConversation(id)
    }
    setIsOpen(false)
    if (onViewChange) {
      onViewChange()
    }
  }, [currentContextId, switchConversation, onViewChange])

  const handleStartRename = useCallback((id: string, currentName: string, e: React.MouseEvent) => {
    e.stopPropagation()
    setRenamingId(id)
    setRenameName(currentName)
  }, [])

  const handleRename = useCallback((id: string) => {
    if (renameName.trim()) {
      renameConversation(id, renameName.trim())
    }
    setRenamingId(null)
  }, [renameName, renameConversation])

  const handleDelete = useCallback((id: string, e: React.MouseEvent) => {
    e.stopPropagation()
    if (conversations.length === 1) {
      return
    }
    deleteConversation(id)
    setIsOpen(false)
  }, [conversations.length, deleteConversation])

  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLInputElement>, id: string) => {
    if (e.key === 'Enter') handleRename(id)
    if (e.key === 'Escape') setRenamingId(null)
  }, [handleRename])

  return (
    <div className="relative w-full md:w-auto my-xs md:my-0" ref={dropdownRef}>
      <ConversationToggleButton
        isSelected={isSelected}
        disabled={disabled}
        isOpen={isOpen}
        isAuthenticated={isAuthenticated}
        currentConversation={currentConversation}
        conversationCount={conversations.length}
        onToggle={handleToggleClick}
      />

      {isAuthenticated && isOpen && !disabled && (
        <div className="absolute left-1/2 -translate-x-1/2 mt-sm z-modal w-[calc(100vw-2rem)] md:w-96">
          <ConversationList
            conversations={conversations}
            currentContextId={currentContextId}
            renamingId={renamingId}
            renameName={renameName}
            onSwitch={handleSwitch}
            onStartRename={handleStartRename}
            onRename={handleRename}
            onDelete={handleDelete}
            onNewConversation={handleNewConversation}
            onRenamingChange={setRenameName}
            onKeyDown={handleKeyDown}
          />
        </div>
      )}
    </div>
  )
})
