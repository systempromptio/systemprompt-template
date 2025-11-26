import { useState, useRef, useEffect } from 'react'
import { useContextStore } from '@/stores/context.store'
import { useAuth } from '@/hooks/useAuth'
import { Button } from '@/components/ui/Button'
import { LogIn } from 'lucide-react'
import { ContextToggleButton } from './ContextToggleButton'
import { ConversationList } from './ConversationList'
import { useKeyboardShortcuts } from '@/lib/accessibility'

export function ContextSelector() {
  const [isOpen, setIsOpen] = useState(false)
  const [isRenaming, setIsRenaming] = useState<string | null>(null)
  const [renameName, setRenameName] = useState('')
  const dropdownRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLInputElement>(null) as React.RefObject<HTMLInputElement>

  const {
    conversationList,
    currentContextId,
    hasReceivedSnapshot,
    createConversation,
    switchConversation,
    renameConversation,
    deleteConversation,
    getCurrentConversation,
  } = useContextStore()

  const { isRealUser, email, showLogin, logout } = useAuth()
  const conversations = conversationList()
  const currentConversation = hasReceivedSnapshot ? getCurrentConversation() : null

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false)
        setIsRenaming(null)
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  // Keyboard shortcuts for context management
  useKeyboardShortcuts([
    {
      key: 'Escape',
      callback: () => {
        setIsOpen(false)
        setIsRenaming(null)
      }
    },
    {
      key: 'n',
      ctrl: true,
      callback: () => {
        if (!isOpen) {
          setIsOpen(true)
        }
        createConversation()
        setIsOpen(false)
      }
    }
  ])

  const handleNewConversation = () => {
    createConversation()
    setIsOpen(false)
  }

  const handleSwitch = (id: string) => {
    if (id !== currentContextId) {
      switchConversation(id)
    }
    setIsOpen(false)
  }

  const handleStartRename = (id: string, currentName: string, e: React.MouseEvent) => {
    e.stopPropagation()
    setIsRenaming(id)
    setRenameName(currentName)
  }

  const handleRename = (id: string) => {
    if (renameName.trim()) {
      renameConversation(id, renameName.trim())
    }
    setIsRenaming(null)
  }

  const handleDelete = (id: string, e: React.MouseEvent) => {
    e.stopPropagation()
    if (conversations.length === 1) {
      return
    }

    deleteConversation(id)
    setIsOpen(false)
  }

  return (
    <div className="relative" ref={dropdownRef}>
      {!isRealUser ? (
        <Button
          variant="secondary"
          size="sm"
          icon={LogIn}
          iconPosition="left"
          onClick={() => showLogin()}
          aria-label="Login to save conversations"
        >
          Login for saved conversations
        </Button>
      ) : (
        <ContextToggleButton
          email={email ?? undefined}
          conversationName={currentConversation?.name || 'No Conversation'}
          isOpen={isOpen}
          onClick={() => setIsOpen(!isOpen)}
          aria-haspopup="listbox"
          aria-expanded={isOpen}
          aria-label="Conversation selector"
        />
      )}

      {isRealUser && isOpen && (
        <div role="listbox" aria-label="Available conversations">
          <ConversationList
            conversations={conversations}
            currentContextId={currentContextId ?? undefined}
            isRenaming={isRenaming}
            renameName={renameName}
            onSelect={handleSwitch}
            onStartRename={handleStartRename}
            onRename={handleRename}
            onDelete={handleDelete}
            onRenamChange={setRenameName}
            onNewConversation={handleNewConversation}
            onLogout={() => {
              logout()
              setIsOpen(false)
            }}
            inputRef={inputRef}
          />
        </div>
      )}
    </div>
  )
}
