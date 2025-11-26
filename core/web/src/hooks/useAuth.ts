/**
 * Hook for authentication state and operations.
 *
 * Provides access to auth state (login status, user info, scopes) and
 * functions for auth-related operations (login, logout, permission checks).
 *
 * @returns Auth state and methods
 *
 * @throws {Error} When logout fails
 *
 * @example
 * ```typescript
 * function ProtectedFeature() {
 *   const { isAuthenticated, requireAuth, logout, isAdmin } = useAuth()
 *
 *   const handleClick = () => {
 *     if (!requireAuth()) return // Shows login modal if needed
 *     // User is authenticated here
 *   }
 *
 *   return (
 *     <div>
 *       {isAuthenticated ? (
 *         <>
 *           <button onClick={handleClick}>Use Feature</button>
 *           {isAdmin() && <button>Admin Panel</button>}
 *           <button onClick={logout}>Logout</button>
 *         </>
 *       ) : (
 *         <button onClick={() => requireAuth()}>Login</button>
 *       )}
 *     </div>
 *   )
 * }
 * ```
 */

import { useCallback } from 'react'
import { useAuthStore } from '@/stores/auth.store'

export function useAuth() {
  const {
    isAuthenticated,
    email,
    username,
    scopes,
    userType,
    isTokenValid,
    clearAuthAndRestoreAnonymous,
    showAuthModal,
    authAgentName,
    openAuthModal,
    closeAuthModal,
    executeAuthCallback,
  } = useAuthStore()

  const requireAuth = useCallback((agentName?: string, onSuccess?: () => void): boolean => {
    if (isAuthenticated && isTokenValid()) {
      onSuccess?.()
      return true
    }

    openAuthModal(agentName, onSuccess)
    return false
  }, [isAuthenticated, isTokenValid, openAuthModal])

  const hasScope = useCallback((scope: string): boolean => {
    return isAuthenticated && isTokenValid() && scopes.includes(scope)
  }, [isAuthenticated, isTokenValid, scopes])

  const isAdmin = useCallback((): boolean => {
    return hasScope('admin')
  }, [hasScope])

  const getPrimaryRole = useCallback((): string => {
    if (!isAuthenticated || !isTokenValid()) return 'Guest'
    if (scopes.includes('admin')) return 'Admin'
    if (scopes.includes('user')) return 'User'
    return userType || 'User'
  }, [isAuthenticated, isTokenValid, scopes, userType])

  const handleAuthSuccess = useCallback(() => {
    executeAuthCallback()
  }, [executeAuthCallback])

  const handleAuthClose = useCallback(() => {
    closeAuthModal()
  }, [closeAuthModal])

  const logout = useCallback(async () => {
    await clearAuthAndRestoreAnonymous()
  }, [clearAuthAndRestoreAnonymous])

  const showLogin = useCallback((onSuccess?: () => void) => {
    openAuthModal(undefined, onSuccess)
  }, [openAuthModal])

  const isRealUser = useCallback((): boolean => {
    return isAuthenticated && isTokenValid() && userType !== 'anon' && email !== null
  }, [isAuthenticated, isTokenValid, userType, email])

  return {
    isAuthenticated: isAuthenticated && isTokenValid(),
    isRealUser: isRealUser(),
    email,
    username,
    scopes,
    userType,
    primaryRole: getPrimaryRole(),
    requireAuth,
    hasScope,
    isAdmin: isAdmin(),
    logout,
    showLogin,
    showAuthModal,
    authAgentName,
    handleAuthSuccess,
    handleAuthClose,
  }
}
