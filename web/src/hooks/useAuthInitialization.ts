/**
 * Authentication initialization hook for anonymous users.
 * Handles the complex logic of setting up anonymous auth with retry logic and global state coordination.
 * @module hooks/useAuthInitialization
 */

import { useRef, useCallback } from 'react'
import { authService } from '@/services/auth.service'
import { useAuthStore } from '@/stores/auth.store'
import { isAuthInitializing, getInitializationPromise, setAuthInitializing } from '@/services/auth-init'

const MAX_RETRIES = 3
const BASE_DELAY = 1000
const MAX_DELAY = 5000

/**
 * Calculate exponential backoff delay for retry attempts.
 * Prevents overwhelming the server with rapid retries.
 * @param retryCount - Number of retries attempted so far
 * @param baseDelay - Base delay in milliseconds (default 1000)
 * @param maxDelay - Maximum delay cap in milliseconds (default 5000)
 * @returns Calculated delay in milliseconds
 * @internal
 */
function calculateBackoffDelay(retryCount: number, baseDelay = BASE_DELAY, maxDelay = MAX_DELAY): number {
  return Math.min(baseDelay * Math.pow(2, retryCount), maxDelay)
}

/**
 * Hook to manage anonymous authentication initialization with exponential backoff retry logic.
 * Coordinates with global singleton to prevent duplicate authentication attempts.
 * Uses local ref tracking and global state to avoid race conditions in concurrent auth attempts.
 * @returns Object with initializeAuth function and error handling callback
 * @internal
 */
export function useAuthInitialization() {
  const isInitializingAuth = useRef(false)
  const retryCount = useRef(0)
  const setAnonymousAuth = useAuthStore((state) => state.setAnonymousAuth)
  const { isTokenValid, accessToken } = useAuthStore()

  /**
   * Attempt anonymous authentication with recursive retry on failure.
   * Checks global singleton to prevent concurrent initialization.
   * Returns early if auth is already valid or already being initialized.
   * @returns Promise resolving to true if auth successful, false otherwise
   * @internal
   */
  const initializeAnonymousAuth = useCallback(async (): Promise<boolean> => {
    if (isAuthInitializing()) {
      const existingPromise = getInitializationPromise()
      if (existingPromise) {
        return await existingPromise
      }
      return false
    }

    if (isInitializingAuth.current) {
      return false
    }

    if (accessToken && isTokenValid()) {
      return true
    }

    isInitializingAuth.current = true

    const initPromise = (async (): Promise<boolean> => {
      try {
        const { token, error } = await authService.generateAnonymousToken()

        if (error) {
          if (retryCount.current < MAX_RETRIES) {
            const delay = calculateBackoffDelay(retryCount.current)
            retryCount.current++
            await new Promise(resolve => setTimeout(resolve, delay))
            isInitializingAuth.current = false
            return await initializeAnonymousAuth()
          }
          isInitializingAuth.current = false
          return false
        }

        if (token) {
          retryCount.current = 0
          setAnonymousAuth(token.access_token, token.user_id, token.session_id, token.expires_in)
          isInitializingAuth.current = false
          return true
        }

        isInitializingAuth.current = false
        return false
      } catch (error) {
        if (retryCount.current < MAX_RETRIES) {
          const delay = calculateBackoffDelay(retryCount.current)
          retryCount.current++
          await new Promise(resolve => setTimeout(resolve, delay))
          isInitializingAuth.current = false
          return await initializeAnonymousAuth()
        }
        isInitializingAuth.current = false
        return false
      }
    })()

    setAuthInitializing(initPromise)
    return await initPromise
  }, [accessToken, isTokenValid, setAnonymousAuth])

  return { initializeAnonymousAuth }
}
