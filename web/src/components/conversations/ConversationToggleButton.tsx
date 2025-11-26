/**
 * Conversation toggle button component.
 *
 * Displays current conversation with open/close chevron.
 *
 * @module components/conversations/ConversationToggleButton
 */

import React, { useCallback } from 'react'
import { MessageSquare, ChevronDown } from 'lucide-react'
import { cn } from '@/lib/utils/cn'
import { useThemeValues } from '@/theme'
import type { Conversation } from '@/stores/context.store'

interface ConversationToggleButtonProps {
  isSelected: boolean
  disabled: boolean
  isOpen: boolean
  isAuthenticated: boolean
  currentConversation: Conversation | null
  conversationCount: number
  onToggle: () => void
}

/**
 * Memoized conversation toggle button.
 * Shows current conversation and opens dropdown when clicked.
 */
export const ConversationToggleButton = React.memo(function ConversationToggleButton({
  isSelected,
  disabled,
  isOpen,
  isAuthenticated,
  currentConversation,
  conversationCount,
  onToggle,
}: ConversationToggleButtonProps) {
  const theme = useThemeValues()

  const getBackgroundStyle = useCallback(() => {
    if (disabled) {
      return { background: 'transparent' }
    }
    if (isSelected) {
      const { card } = theme.components
      return {
        background: `linear-gradient(135deg, ${card.gradient.start}, ${card.gradient.mid}, ${card.gradient.end})`,
      }
    }
    return { background: 'transparent' }
  }, [disabled, isSelected, theme])

  return (
    <button
      onClick={disabled ? undefined : onToggle}
      disabled={disabled}
      className={cn(
        'group relative flex items-center gap-sm',
        'px-lg py-sm md:py-md',
        'border transition-all duration-fast',
        'focus:outline-none',
        'min-h-[44px] md:min-h-[60px]',
        'w-full md:w-auto',
        disabled
          ? 'border-text-disabled/30 opacity-40 cursor-not-allowed grayscale'
          : cn(
              'focus:ring-2 focus:ring-primary focus:ring-offset-2',
              isSelected
                ? 'border-primary text-white'
                : 'border-primary/30 text-text-secondary hover:border-primary/60 hover:scale-105'
            )
      )}
      style={{
        borderRadius: `${theme.components.card.borderRadius.default}px`,
        borderTopRightRadius: `${theme.components.card.borderRadius.topRight}px`,
        ...getBackgroundStyle(),
      }}
    >
      <MessageSquare
        className={cn(
          'w-5 h-5 flex-shrink-0',
          disabled
            ? 'text-text-disabled'
            : isSelected
            ? 'text-white'
            : 'text-primary'
        )}
      />

      <div className="flex flex-col items-start min-w-0 flex-1">
        {currentConversation ? (
          <>
            <span className={cn(
              'text-sm font-heading font-medium uppercase tracking-wide truncate max-w-full',
              disabled
                ? 'text-text-disabled'
                : isSelected
                ? 'text-white'
                : 'text-text-primary'
            )}>
              {currentConversation.name}
            </span>
            <span className={cn(
              'text-xs font-body',
              disabled
                ? 'text-text-disabled'
                : isSelected
                ? 'text-white/70'
                : 'text-text-secondary'
            )}>
              {currentConversation.messageCount} {currentConversation.messageCount === 1 ? 'message' : 'messages'}
            </span>
          </>
        ) : (
          <span className={cn(
            'hidden sm:inline text-sm font-heading font-medium uppercase tracking-wide',
            disabled
              ? 'text-text-disabled'
              : isSelected
              ? 'text-white'
              : 'text-text-primary'
          )}>
            Conversations
          </span>
        )}
      </div>

      {!currentConversation && (
        <span
          className={cn(
            'flex items-center justify-center min-w-[24px] h-5 px-xs',
            'rounded-full text-xs font-body font-semibold',
            disabled
              ? 'bg-text-disabled/10 text-text-disabled'
              : isSelected
              ? 'bg-white/20 text-white'
              : 'bg-primary/10 text-primary'
          )}
        >
          {conversationCount}
        </span>
      )}

      {isAuthenticated && !disabled && (
        <ChevronDown
          className={cn(
            'w-3.5 h-3.5 transition-transform ml-xs',
            isSelected ? 'text-white/70' : 'text-primary/70',
            isOpen ? 'rotate-180' : ''
          )}
        />
      )}
    </button>
  )
})
