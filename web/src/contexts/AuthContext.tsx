import { createContext, useContext, useState, useEffect, useCallback, type ReactNode } from 'react'

export interface User {
  id: string
  email: string
  name?: string
  roles?: string[]
  metadata?: Record<string, unknown>
}

export interface AuthState {
  user: User | null
  token: string | null
  isAuthenticated: boolean
  isLoading: boolean
  error: string | null
}

export interface AuthContextValue extends AuthState {
  login: (email: string, password: string) => Promise<void>
  logout: () => Promise<void>
  register: (email: string, password: string, name?: string) => Promise<void>
  refreshToken: () => Promise<void>
  updateUser: (updates: Partial<User>) => void
  clearError: () => void
}

const AuthContext = createContext<AuthContextValue | undefined>(undefined)

export function useAuth() {
  const context = useContext(AuthContext)
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider')
  }
  return context
}

export function useOptionalAuth() {
  return useContext(AuthContext)
}

interface AuthProviderProps {
  children: ReactNode
  apiBaseUrl?: string
  tokenKey?: string
  onAuthChange?: (isAuthenticated: boolean) => void
}

export function AuthProvider({
  children,
  apiBaseUrl = '/api/v1',
  tokenKey = 'auth_token',
  onAuthChange,
}: AuthProviderProps) {
  const [state, setState] = useState<AuthState>({
    user: null,
    token: null,
    isAuthenticated: false,
    isLoading: true,
    error: null,
  })

  const setToken = useCallback(
    (token: string | null) => {
      if (token) {
        localStorage.setItem(tokenKey, token)
      } else {
        localStorage.removeItem(tokenKey)
      }
    },
    [tokenKey]
  )

  const setError = useCallback((error: string | null) => {
    setState((prev) => ({ ...prev, error, isLoading: false }))
  }, [])

  const clearError = useCallback(() => {
    setState((prev) => ({ ...prev, error: null }))
  }, [])

  const parseToken = useCallback((token: string): User | null => {
    try {
      const payload = token.split('.')[1]
      if (!payload) return null

      const decoded = JSON.parse(atob(payload))

      return {
        id: decoded.sub || decoded.user_id || '',
        email: decoded.email || '',
        name: decoded.name,
        roles: decoded.roles || [],
        metadata: decoded.metadata,
      }
    } catch {
      return null
    }
  }, [])

  const login = useCallback(
    async (email: string, password: string) => {
      setState((prev) => ({ ...prev, isLoading: true, error: null }))

      try {
        const response = await fetch(`${apiBaseUrl}/auth/login`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ email, password }),
        })

        if (!response.ok) {
          const error = await response.json()
          throw new Error(error.message || 'Login failed')
        }

        const data = await response.json()
        const { token } = data

        setToken(token)
        const user = parseToken(token)

        setState({
          user,
          token,
          isAuthenticated: true,
          isLoading: false,
          error: null,
        })

        onAuthChange?.(true)
      } catch (err) {
        const message = err instanceof Error ? err.message : 'Login failed'
        setError(message)
        throw err
      }
    },
    [apiBaseUrl, onAuthChange, parseToken, setError, setToken]
  )

  const register = useCallback(
    async (email: string, password: string, name?: string) => {
      setState((prev) => ({ ...prev, isLoading: true, error: null }))

      try {
        const response = await fetch(`${apiBaseUrl}/auth/register`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ email, password, name }),
        })

        if (!response.ok) {
          const error = await response.json()
          throw new Error(error.message || 'Registration failed')
        }

        const data = await response.json()
        const { token } = data

        setToken(token)
        const user = parseToken(token)

        setState({
          user,
          token,
          isAuthenticated: true,
          isLoading: false,
          error: null,
        })

        onAuthChange?.(true)
      } catch (err) {
        const message = err instanceof Error ? err.message : 'Registration failed'
        setError(message)
        throw err
      }
    },
    [apiBaseUrl, onAuthChange, parseToken, setError, setToken]
  )

  const logout = useCallback(async () => {
    try {
      if (state.token) {
        await fetch(`${apiBaseUrl}/auth/logout`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Authorization: `Bearer ${state.token}`,
          },
        })
      }
    } catch {
    } finally {
      setToken(null)
      setState({
        user: null,
        token: null,
        isAuthenticated: false,
        isLoading: false,
        error: null,
      })
      onAuthChange?.(false)
    }
  }, [apiBaseUrl, onAuthChange, setToken, state.token])

  const refreshToken = useCallback(async () => {
    if (!state.token) {
      setError('No token to refresh')
      return
    }

    try {
      const response = await fetch(`${apiBaseUrl}/auth/refresh`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${state.token}`,
        },
      })

      if (!response.ok) {
        throw new Error('Token refresh failed')
      }

      const data = await response.json()
      const { token } = data

      setToken(token)
      const user = parseToken(token)

      setState((prev) => ({
        ...prev,
        user,
        token,
        isAuthenticated: true,
      }))
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Token refresh failed'
      setError(message)
      await logout()
    }
  }, [apiBaseUrl, logout, parseToken, setError, setToken, state.token])

  const updateUser = useCallback((updates: Partial<User>) => {
    setState((prev) => ({
      ...prev,
      user: prev.user ? { ...prev.user, ...updates } : null,
    }))
  }, [])

  useEffect(() => {
    const token = localStorage.getItem(tokenKey)

    if (token) {
      const user = parseToken(token)

      if (user) {
        setState({
          user,
          token,
          isAuthenticated: true,
          isLoading: false,
          error: null,
        })
        onAuthChange?.(true)
      } else {
        localStorage.removeItem(tokenKey)
        setState({
          user: null,
          token: null,
          isAuthenticated: false,
          isLoading: false,
          error: null,
        })
      }
    } else {
      setState((prev) => ({ ...prev, isLoading: false }))
    }
  }, [onAuthChange, parseToken, tokenKey])

  const value: AuthContextValue = {
    ...state,
    login,
    logout,
    register,
    refreshToken,
    updateUser,
    clearError,
  }

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}

export function RequireAuth({
  children,
  fallback,
  redirectTo,
}: {
  children: ReactNode
  fallback?: ReactNode
  redirectTo?: string
}) {
  const { isAuthenticated, isLoading } = useAuth()

  useEffect(() => {
    if (!isLoading && !isAuthenticated && redirectTo) {
      window.location.href = redirectTo
    }
  }, [isAuthenticated, isLoading, redirectTo])

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-text-secondary">Loading...</div>
      </div>
    )
  }

  if (!isAuthenticated) {
    if (fallback) {
      return <>{fallback}</>
    }

    return (
      <div className="min-h-screen bg-background flex items-center justify-center p-8">
        <div className="bg-surface border border-border rounded-lg p-8 max-w-md w-full text-center">
          <h2 className="text-xl font-bold text-text-primary mb-4">Authentication Required</h2>
          <p className="text-text-secondary mb-6">
            You need to be logged in to access this content.
          </p>
          <button
            onClick={() => (window.location.href = redirectTo || '/login')}
            className="px-6 py-3 bg-primary text-white rounded-lg hover:bg-primary/90 transition-fast font-medium"
          >
            Go to Login
          </button>
        </div>
      </div>
    )
  }

  return <>{children}</>
}
