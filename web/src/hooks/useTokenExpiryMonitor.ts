import { useEffect, useRef } from 'react'
import { useAuthStore } from '@/stores/auth.store'
import { useContextStore, CONTEXT_STATE } from '@/stores/context.store'
import { authService } from '@/services/auth.service'
import { logger } from '@/lib/logger'

import { TIMING } from '@/constants/timing'

/**
 * Monitors authentication token expiry and automatically refreshes tokens.
 *
 * Checks token expiry on a 5-minute interval and refreshes both anonymous and
 * authenticated user tokens when they expire. Authenticated users are refreshed
 * using their refresh token to maintain session continuity.
 *
 * @returns void - Handles token management as side effect
 *
 * @example
 * ```typescript
 * useTokenExpiryMonitor() // Auto-runs in app root
 * ```
 */
export function useTokenExpiryMonitor() {
  const { isAuthenticated, tokenExpiry, userType, refreshToken, clearAuth, setAnonymousAuth, updateTokens } = useAuthStore()
  const intervalRef = useRef<NodeJS.Timeout | null>(null)
  const isRefreshing = useRef(false)

  useEffect(() => {
    if (!isAuthenticated || !tokenExpiry) {
      if (intervalRef.current) {
        clearInterval(intervalRef.current)
        intervalRef.current = null
      }
      return
    }

    const checkTokenExpiry = async () => {
      const now = Date.now()
      const timeUntilExpiry = tokenExpiry - now

      if (timeUntilExpiry <= 0) {
        logger.debug('Token expired', undefined, 'useTokenExpiryMonitor')

        if (userType === 'anon' && !isRefreshing.current) {
          logger.debug('Refreshing anonymous token', undefined, 'useTokenExpiryMonitor')
          isRefreshing.current = true

          try {
            const { token, error } = await authService.generateAnonymousToken()

            if (error) {
              logger.error('Failed to refresh anonymous token', new Error(String(error)), 'useTokenExpiryMonitor')
              clearAuth()
              useContextStore.setState({ conversations: new Map(), currentContextId: CONTEXT_STATE.LOADING })
            } else if (token) {
              logger.debug('Anonymous token refreshed successfully', undefined, 'useTokenExpiryMonitor')
              setAnonymousAuth(
                token.access_token,
                token.user_id,
                token.session_id,
                token.expires_in
              )
            }
          } catch (error) {
            logger.error('Error refreshing anonymous token', error, 'useTokenExpiryMonitor')
            clearAuth()
            useContextStore.setState({ conversations: new Map(), currentContextId: CONTEXT_STATE.LOADING })
          } finally {
            isRefreshing.current = false
          }
        } else if (userType !== 'anon' && refreshToken && !isRefreshing.current) {
          logger.debug('Refreshing authenticated user token', undefined, 'useTokenExpiryMonitor')
          isRefreshing.current = true

          try {
            const { token, error } = await authService.refreshAccessToken(refreshToken)

            if (error || !token) {
              logger.error('Failed to refresh authenticated token', new Error(String(error)), 'useTokenExpiryMonitor')
              clearAuth()
              useContextStore.setState({ conversations: new Map(), currentContextId: CONTEXT_STATE.LOADING })
            } else {
              logger.debug('Authenticated token refreshed successfully', undefined, 'useTokenExpiryMonitor')
              updateTokens(
                token.access_token,
                token.refresh_token || refreshToken,
                token.expires_in
              )
            }
          } catch (error) {
            logger.error('Error refreshing authenticated token', error, 'useTokenExpiryMonitor')
            clearAuth()
            useContextStore.setState({ conversations: new Map(), currentContextId: CONTEXT_STATE.LOADING })
          } finally {
            isRefreshing.current = false
          }
        } else {
          logger.debug('Token expired but cannot refresh (no refresh token), clearing auth', undefined, 'useTokenExpiryMonitor')
          clearAuth()
          useContextStore.setState({ conversations: new Map(), currentContextId: CONTEXT_STATE.LOADING })
        }
      } else if (timeUntilExpiry <= TIMING.TOKEN_REFRESH_THRESHOLD && userType !== 'anon' && refreshToken) {
        logger.debug('Token expires soon for authenticated user, will auto-refresh when expired', { minutesLeft: Math.round(timeUntilExpiry / 60000) }, 'useTokenExpiryMonitor')
      }
    }

    checkTokenExpiry()

    intervalRef.current = setInterval(checkTokenExpiry, TIMING.TOKEN_CHECK_INTERVAL)

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current)
        intervalRef.current = null
      }
    }
  }, [isAuthenticated, tokenExpiry, userType, refreshToken, clearAuth, setAnonymousAuth, updateTokens])
}
