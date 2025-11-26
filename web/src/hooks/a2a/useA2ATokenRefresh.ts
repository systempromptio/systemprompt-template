/**
 * Hook for managing A2A client token refresh.
 *
 * Handles automatic token refresh on expiry and provides
 * manual refresh capability for token update scenarios.
 *
 * @module hooks/a2a/useA2ATokenRefresh
 */

import { useState, useCallback, useEffect } from 'react'
import { useAuthStore } from '@/stores/auth.store'
import { logger } from '@/lib/logger'

/**
 * Token refresh hook return value.
 */
interface UseA2ATokenRefreshReturn {
  /**
   * Current token value (may be null if not authenticated)
   */
  token: string | null

  /**
   * Whether token is currently being refreshed
   */
  isRefreshing: boolean

  /**
   * Manually trigger token refresh
   */
  refresh: () => Promise<void>

  /**
   * Error from last refresh attempt, if any
   */
  error: string | null

  /**
   * Clear current error state
   */
  clearError: () => void
}

/**
 * Manages A2A client authentication token lifecycle.
 *
 * Automatically monitors auth store for token availability and provides
 * manual refresh capability for token expiry scenarios.
 *
 * @returns Token refresh state and controls
 *
 * @example
 * ```typescript
 * function A2AClientComponent() {
 *   const { token, refresh, isRefreshing } = useA2ATokenRefresh()
 *
 *   const handleSend = async () => {
 *     if (!token) {
 *       await refresh()
 *     }
 *     // Use token for API call
 *   }
 * }
 * ```
 *
 * @throws {Error} When token refresh fails
 */
export function useA2ATokenRefresh(): UseA2ATokenRefreshReturn {
  const getAuthHeader = useAuthStore((state) => state.getAuthHeader)
  const accessToken = useAuthStore((state) => state.accessToken)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const extractToken = useCallback((header: string | null): string | null => {
    if (!header) return null
    return header.replace('Bearer ', '')
  }, [])

  const authHeader = getAuthHeader()
  const token = extractToken(authHeader) || extractToken(accessToken ? `Bearer ${accessToken}` : null)

  const refresh = useCallback(async () => {
    try {
      setIsRefreshing(true)
      setError(null)

      const { authService } = await import('@/services/auth.service')
      const { token: refreshedToken, error: refreshError } = await authService.generateAnonymousToken()

      if (refreshError || !refreshedToken) {
        const message = refreshError || 'Token refresh failed'
        setError(String(message))
        logger.error('Token refresh failed', new Error(String(message)), 'useA2ATokenRefresh')
        throw new Error(String(message))
      }

      logger.debug('A2A token refreshed', undefined, 'useA2ATokenRefresh')
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Token refresh failed'
      setError(message)
      logger.error('Token refresh error', err, 'useA2ATokenRefresh')
      throw err
    } finally {
      setIsRefreshing(false)
    }
  }, [])

  const clearError = useCallback(() => {
    setError(null)
  }, [])

  useEffect(() => {
    if (!token) {
      logger.debug('No A2A token available', undefined, 'useA2ATokenRefresh')
    }
  }, [token])

  return { token, isRefreshing, refresh, error, clearError }
}
