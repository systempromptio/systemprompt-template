/**
 * Hook for automatic initialization of default conversation context.
 *
 * Ensures that a default conversation is created when the user is authenticated
 * but no conversations exist yet. Runs automatically on auth state changes and
 * snapshot receipt.
 *
 * This hook requires no parameters and is typically mounted at the app root level
 * to initialize the context layer on authentication.
 *
 * @throws {Error} When creating default context fails (logged but not thrown)
 *
 * @example
 * ```typescript
 * function App() {
 *   // Mount at app root to ensure context is initialized
 *   useContextInit()
 *
 *   const { conversations } = useContextStore()
 *
 *   return (
 *     <div>
 *       <ConversationList conversations={conversations} />
 *       <ChatView />
 *     </div>
 *   )
 * }
 * ```
 */

import { useEffect, useRef } from 'react'
import { useAuthStore } from '@/stores/auth.store'
import { useContextStore, CONTEXT_STATE } from '@/stores/context.store'
import { logger } from '@/lib/logger'

export function useContextInit() {
  const accessToken = useAuthStore((state) => state.accessToken)
  const isTokenValid = useAuthStore((state) => state.isTokenValid)
  const currentContextId = useContextStore((state) => state.currentContextId)
  const conversations = useContextStore((state) => state.conversations)
  const hasReceivedSnapshot = useContextStore((state) => state.hasReceivedSnapshot)
  const createConversation = useContextStore((state) => state.createConversation)
  const isCreatingContext = useRef(false)

  useEffect(() => {
    const ensureContext = async () => {
      if (isCreatingContext.current) {
        return
      }

      const hasValidToken = accessToken && isTokenValid()
      const isLoading = currentContextId === CONTEXT_STATE.LOADING
      const hasNoContexts = conversations.size === 0

      if (hasValidToken && isLoading && hasReceivedSnapshot && hasNoContexts) {
        isCreatingContext.current = true

        try {
          await createConversation('Default Conversation')
        } catch (error) {
          logger.error('Failed to create default context', error, 'useContextInit')
        } finally {
          isCreatingContext.current = false
        }
      }
    }

    ensureContext()
  }, [accessToken, isTokenValid, currentContextId, conversations.size, hasReceivedSnapshot])
}
