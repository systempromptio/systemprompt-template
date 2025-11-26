import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { extractScopesFromJWT, extractUserTypeFromJWT, extractUsernameFromJWT, extractSessionIdFromJWT } from '@/utils/jwt'
import { resetAllStores, clearUserLocalStorage } from './reset'
import { UIStateKey } from '@/constants'

/**
 * Authenticated user with complete session information.
 * All fields are guaranteed to be non-null in this type.
 */
export interface AuthUser {
  userId: string
  email: string | null
  username: string
  sessionId: string
  userType: string
  scopes: readonly string[]
  accessToken: string
  refreshToken: string | null
  tokenExpiry: number
}

interface AuthState {
  isAuthenticated: boolean
  email: string | null
  userId: string | null
  sessionId: string | null
  username: string | null
  scopes: readonly string[]
  userType: string | null
  accessToken: string | null
  refreshToken: string | null
  tokenExpiry: number | null

  showAuthModal: boolean
  authAgentName: string | undefined
  authCallback: (() => void) | null

  setAuth: (email: string, userId: string, accessToken: string, refreshToken: string | null, expiresIn: number) => void
  updateTokens: (accessToken: string, refreshToken: string | null, expiresIn: number) => void
  setAnonymousAuth: (accessToken: string, userId: string, sessionId: string, expiresIn: number) => void
  clearAuth: () => void
  clearAuthAndRestoreAnonymous: () => Promise<void>
  isTokenValid: () => boolean
  getAuthHeader: () => string | null

  openAuthModal: (agentName?: string, callback?: () => void) => void
  closeAuthModal: () => void
  executeAuthCallback: () => void
}

const TOKEN_EXPIRY_BUFFER_MS = 30000

const extractTokenData = (accessToken: string) => ({
  sessionId: extractSessionIdFromJWT(accessToken),
  username: extractUsernameFromJWT(accessToken),
  scopes: extractScopesFromJWT(accessToken) as readonly string[],
  userType: extractUserTypeFromJWT(accessToken),
})

const clearAuthState = () => ({
  isAuthenticated: false,
  email: null,
  userId: null,
  sessionId: null,
  username: null,
  scopes: [] as readonly string[],
  userType: null,
  accessToken: null,
  refreshToken: null,
  tokenExpiry: null,
})

const handleUserSwitch = (previousUserId: string | null) => {
  clearUserLocalStorage(previousUserId || undefined)
  resetAllStores()
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      isAuthenticated: false,
      email: null,
      userId: null,
      sessionId: null,
      username: null,
      scopes: [] as readonly string[],
      userType: null,
      accessToken: null,
      refreshToken: null,
      tokenExpiry: null,

      showAuthModal: false,
      authAgentName: undefined,
      authCallback: null,

      /**
       * Authenticate with email and access token.
       * Extracts claims from JWT and handles user switching if needed.
       * @param email - User email address
       * @param userId - User ID
       * @param accessToken - JWT access token
       * @param refreshToken - Refresh token for obtaining new access tokens
       * @param expiresIn - Token expiration time in seconds
       */
      setAuth: (email, userId, accessToken, refreshToken, expiresIn) => {
        const previousUserId = get().userId
        const tokenData = extractTokenData(accessToken)

        set({
          isAuthenticated: true,
          email,
          userId,
          ...tokenData,
          accessToken,
          refreshToken,
          tokenExpiry: Date.now() + (expiresIn * 1000),
        })

        if (previousUserId !== userId) {
          handleUserSwitch(previousUserId)
        }
      },

      /**
       * Update access and refresh tokens without changing other auth state.
       * Used when refreshing tokens to extend the session.
       * @param accessToken - New JWT access token
       * @param refreshToken - New refresh token
       * @param expiresIn - Token expiration time in seconds
       */
      updateTokens: (accessToken, refreshToken, expiresIn) => {
        const tokenData = extractTokenData(accessToken)
        set({
          ...tokenData,
          accessToken,
          refreshToken,
          tokenExpiry: Date.now() + (expiresIn * 1000),
        })
      },

      /**
       * Authenticate as anonymous user with session token.
       * Handles user switching if transitioning from authenticated state.
       * @param accessToken - JWT session token
       * @param userId - Anonymous user ID
       * @param sessionId - Session identifier
       * @param expiresIn - Token expiration time in seconds
       */
      setAnonymousAuth: (accessToken, userId, sessionId, expiresIn) => {
        const previousUserId = get().userId
        const tokenData = extractTokenData(accessToken)

        set({
          isAuthenticated: true,
          email: null,
          userId,
          sessionId,
          username: tokenData.username || 'Anonymous',
          scopes: tokenData.scopes,
          userType: tokenData.userType || 'anon',
          accessToken,
          tokenExpiry: Date.now() + (expiresIn * 1000),
        })

        if (previousUserId !== userId) {
          handleUserSwitch(previousUserId)
        }
      },

      /**
       * Clear all authentication state and reset app.
       * Clears local storage and resets all stores.
       */
      clearAuth: () => {
        const previousUserId = get().userId
        set(clearAuthState())
        handleUserSwitch(previousUserId)
      },

      /**
       * Logout and restore anonymous session.
       * Clears authenticated state, resets stores, and generates new anonymous token.
       * Safe to fail silently if anonymous token generation fails.
       */
      clearAuthAndRestoreAnonymous: async () => {
        const previousUserId = get().userId
        set(clearAuthState())
        handleUserSwitch(previousUserId)

        const { authService } = await import('@/services/auth.service')
        const { token, error } = await authService.generateAnonymousToken()

        if (error || !token) return

        const tokenData = extractTokenData(token.access_token)
        set({
          isAuthenticated: true,
          email: null,
          userId: token.user_id,
          sessionId: token.session_id,
          username: tokenData.username || 'Anonymous',
          scopes: tokenData.scopes,
          userType: tokenData.userType || 'anon',
          accessToken: token.access_token,
          tokenExpiry: Date.now() + (token.expires_in * 1000),
        })
      },

      /**
       * Check if current access token is still valid.
       * Returns false if token is missing or expired (including buffer).
       * @returns True if token exists and has not expired
       */
      isTokenValid: () => {
        const state = get()
        if (!state.accessToken || !state.tokenExpiry) return false
        return Date.now() < (state.tokenExpiry - TOKEN_EXPIRY_BUFFER_MS)
      },

      /**
       * Get Bearer token for API requests.
       * Only returns token if authenticated, token exists, and token is valid.
       * @returns Bearer token string or null if invalid
       */
      getAuthHeader: () => {
        const state = get()
        return state.isAuthenticated && state.accessToken && state.isTokenValid()
          ? `Bearer ${state.accessToken}`
          : null
      },

      /**
       * Open authentication modal with optional agent context and callback.
       * @param agentName - Optional agent requesting authentication
       * @param callback - Optional callback to execute after successful auth
       */
      openAuthModal: (agentName, callback) => {
        set({
          showAuthModal: true,
          authAgentName: agentName,
          authCallback: callback || null,
        })
      },

      /**
       * Close authentication modal and clear context.
       */
      closeAuthModal: () => {
        set({
          showAuthModal: false,
          authAgentName: undefined,
          authCallback: null,
        })
      },

      /**
       * Execute pending auth callback and close modal.
       * Safe to call if no callback is pending.
       */
      executeAuthCallback: () => {
        const { authCallback } = get()
        authCallback?.()
        get().closeAuthModal()
      },
    }),
    {
      name: UIStateKey.AUTH_STORAGE,
      partialize: (state) => ({
        isAuthenticated: state.isAuthenticated,
        email: state.email,
        userId: state.userId,
        sessionId: state.sessionId,
        username: state.username,
        scopes: state.scopes,
        userType: state.userType,
        accessToken: state.accessToken,
        refreshToken: state.refreshToken,
        tokenExpiry: state.tokenExpiry,
      }),
      onRehydrateStorage: () => (state) => {
        if (!state) return

        if (!Array.isArray(state.scopes)) {
          state.scopes = []
        }

        const BUFFER_MS = 30000
        if (state.tokenExpiry && Date.now() >= (state.tokenExpiry - BUFFER_MS)) {
          state.clearAuth()
        }
      },
    }
  )
)

/**
 * Selectors for reading auth state with type safety.
 * All selectors return computed or derived values without side effects.
 */
export const authSelectors = {
  /**
   * Get current authenticated user with all claims, or null if not authenticated.
   * Validates all required fields are present before returning user object.
   * @param state - Current auth state
   * @returns Authenticated user or null
   */
  getCurrentUser: (state: AuthState): AuthUser | null => {
    if (!state.isAuthenticated || !state.userId || !state.accessToken || !state.tokenExpiry) {
      return null
    }
    return {
      userId: state.userId,
      email: state.email,
      username: state.username ?? 'Anonymous',
      sessionId: state.sessionId ?? '',
      userType: state.userType ?? 'user',
      scopes: state.scopes,
      accessToken: state.accessToken,
      refreshToken: state.refreshToken,
      tokenExpiry: state.tokenExpiry,
    }
  },

  /**
   * Check if user is currently authenticated.
   * @param state - Current auth state
   * @returns True if authenticated
   */
  isAuthenticated: (state: AuthState): boolean => state.isAuthenticated,

  /**
   * Check if access token is valid and not expired.
   * Includes expiry buffer for safety margin.
   * @param state - Current auth state
   * @returns True if token is valid
   */
  hasValidToken: (state: AuthState): boolean => {
    if (!state.isAuthenticated || !state.accessToken || !state.tokenExpiry) {
      return false
    }
    return Date.now() < (state.tokenExpiry - TOKEN_EXPIRY_BUFFER_MS)
  },

  /**
   * Get user's OAuth scopes/permissions.
   * @param state - Current auth state
   * @returns Array of scope strings
   */
  getScopes: (state: AuthState): readonly string[] => state.scopes,

  /**
   * Get current user ID.
   * @param state - Current auth state
   * @returns User ID or null if not authenticated
   */
  getUserId: (state: AuthState): string | null => state.userId,

  /**
   * Get current username.
   * @param state - Current auth state
   * @returns Username or null if not authenticated
   */
  getUsername: (state: AuthState): string | null => state.username,

  /**
   * Get current user type/role.
   * @param state - Current auth state
   * @returns User type or null if not authenticated
   */
  getUserType: (state: AuthState): string | null => state.userType,
}