import { useCallback } from 'react'
import { useAuthStore } from '@/stores/auth.store'
import { useContextStore } from '@/stores/context.store'
import { logger } from '@/lib/logger'

/**
 * Hook for managing SSE token refresh with exponential backoff.
 *
 * Handles:
 * - Anonymous token refresh for 401 errors
 * - Auth state management
 * - Error logging
 *
 * @param onDisconnected - Callback when disconnection required
 * @returns {Object} Token management functions
 * @returns {Function} handleTokenRefresh - Attempt to refresh token
 *
 * @example
 * ```typescript
 * const { handleTokenRefresh } = useSSETokenManagement(() => disconnect())
 * const success = await handleTokenRefresh()
 * ```
 */
export function useSSETokenManagement(onDisconnected?: () => void) {
  const handleTokenRefresh = useCallback(async (): Promise<boolean> => {
    try {
      const { authService } = await import('@/services/auth.service')
      const { token, error: tokenError } = await authService.generateAnonymousToken()

      if (tokenError || !token) {
        logger.error('Failed to refresh token', tokenError, 'useSSETokenManagement')
        useAuthStore.getState().clearAuth()
        useContextStore.getState().setSSEStatus('disconnected')
        onDisconnected?.()
        return false
      }

      useAuthStore.getState().setAnonymousAuth(
        token.access_token,
        token.user_id,
        token.session_id,
        token.expires_in
      )

      logger.info('Token refreshed, reconnecting', undefined, 'useSSETokenManagement')
      return true
    } catch (err) {
      logger.error('Error refreshing token', err, 'useSSETokenManagement')
      return false
    }
  }, [onDisconnected])

  return { handleTokenRefresh }
}
