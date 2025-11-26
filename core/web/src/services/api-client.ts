/**
 * Centralized HTTP API Client
 *
 * Provides type-safe HTTP operations with automatic error handling, token refresh,
 * rate limit handling, and response validation. Designed to handle all REST API
 * communication for the application.
 *
 * Features:
 * - Automatic token refresh on 401 errors
 * - Rate limit handling with user-friendly messaging
 * - Response validation (JSON, content-type, status codes)
 * - Authorization header injection
 * - Comprehensive error reporting
 *
 * @example
 * ```typescript
 * const { data, error, status } = await apiClient.get<Task[]>('/tasks')
 * if (error) { console.error(error); return }
 * console.log(data) // Task[]
 * ```
 */

import { logger } from '@/lib/logger'

/**
 * Error response from API
 */
interface ApiError {
  code: string
  message: string
  details?: any
}

/**
 * Options for HTTP request
 */
interface RequestOptions {
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH'
  headers?: Record<string, string>
  body?: any
  authToken?: string | null
  _retryCount?: number
}

/**
 * Response from API operation
 */
interface ApiResponse<T> {
  data?: T
  error?: string
  status?: number
}

/**
 * Centralized HTTP client for all API communication
 * @internal
 */
class ApiClient {
  private baseUrl: string
  private isRefreshingToken: boolean = false
  private refreshPromise: Promise<string | null> | null = null

  /**
   * Initialize API client
   * Combines VITE_API_BASE_HOST and VITE_API_BASE_PATH for the full base URL
   * Falls back to individual variables, then to legacy VITE_API_BASE_URL for backwards compatibility
   * Final fallback: /api/v1/core
   */
  constructor(baseUrl?: string) {
    if (baseUrl) {
      this.baseUrl = baseUrl
    } else {
      // Try new separated variables first (host + path)
      const host = import.meta.env.VITE_API_BASE_HOST
      const path = import.meta.env.VITE_API_BASE_PATH

      if (host && path) {
        this.baseUrl = `${host}${path}`
      } else if (host) {
        // Fallback: host only with default path
        this.baseUrl = `${host}/api/v1/core`
      } else if (path) {
        // Fallback: path only (relative)
        this.baseUrl = path
      } else {
        // Legacy fallback: combined variable
        this.baseUrl = import.meta.env.VITE_API_BASE_URL || '/api/v1/core'
      }
    }
  }

  /**
   * Refresh authentication token (with deduplication to avoid multiple concurrent refreshes)
   * @internal
   */
  private async refreshToken(): Promise<string | null> {
    if (this.isRefreshingToken && this.refreshPromise) {
      return this.refreshPromise
    }

    this.isRefreshingToken = true
    this.refreshPromise = this.performTokenRefresh()

    try {
      return await this.refreshPromise
    } finally {
      this.isRefreshingToken = false
      this.refreshPromise = null
    }
  }

  /**
   * Execute token refresh for anonymous users
   * @internal
   */
  private async performTokenRefresh(): Promise<string | null> {
    try {
      const { useAuthStore } = await import('@/stores/auth.store')
      await import('./auth.service')

      const userType = useAuthStore.getState().userType

      if (userType === 'anon') {
        logger.debug('Refreshing anonymous token due to 401', undefined, 'api-client')
        return await this.refreshAnonymousToken()
      }

      logger.debug('Authenticated user token expired, clearing auth', undefined, 'api-client')
      useAuthStore.getState().clearAuth()
      return null
    } catch (error) {
      logger.error('Error during token refresh', error, 'api-client')
      return null
    }
  }

  /**
   * Generate new anonymous token and update auth state
   * @internal
   */
  private async refreshAnonymousToken(): Promise<string | null> {
    try {
      const { useAuthStore } = await import('@/stores/auth.store')
      const { authService } = await import('./auth.service')

      const { token, error } = await authService.generateAnonymousToken()

      if (error || !token) {
        logger.error('Failed to refresh token', error, 'api-client')
        useAuthStore.getState().clearAuth()
        return null
      }

      useAuthStore.getState().setAnonymousAuth(
        token.access_token,
        token.user_id,
        token.session_id,
        token.expires_in
      )

      return `Bearer ${token.access_token}`
    } catch (error) {
      logger.error('Error generating anonymous token', error, 'api-client')
      return null
    }
  }

  /**
   * Extract error message from response
   * @internal
   */
  private async extractErrorText(response: Response): Promise<string> {
    const contentType = response.headers.get('content-type')
    const isJson = contentType?.includes('application/json')

    try {
      if (isJson) {
        const data = await response.json()
        return data.message || ''
      }
      return await response.text()
    } catch {
      return ''
    }
  }

  /**
   * Handle 401 unauthorized response with token refresh attempt
   * @internal
   */
  private async handle401Error<T>(
    response: Response,
    endpoint: string,
    options: RequestOptions,
  ): Promise<ApiResponse<T>> {
    const responseText = await this.extractErrorText(response)

    if (responseText.includes('Invalid or expired JWT token') && (options._retryCount || 0) === 0) {
      logger.debug('Detected expired token, attempting refresh', undefined, 'api-client')
      const newToken = await this.refreshToken()

      if (newToken) {
        logger.debug('Token refreshed, retrying request', undefined, 'api-client')
        return this.request<T>(endpoint, {
          ...options,
          authToken: newToken,
          _retryCount: 1,
        })
      }
    }

    return {
      error: responseText || 'Unauthorized. Please log in again.',
      status: 401,
    }
  }

  /**
   * Handle non-OK HTTP responses
   * @internal
   */
  private async handleErrorResponse<T>(response: Response): Promise<ApiResponse<T>> {
    const contentType = response.headers.get('content-type')
    const isJson = contentType?.includes('application/json')

    if (isJson) {
      const data = await response.json()
      const error = data as ApiError
      return {
        error: error.message || `Request failed with status ${response.status}`,
        status: response.status,
      }
    }

    const text = await response.text()
    return {
      error: text || `Request failed with status ${response.status}`,
      status: response.status,
    }
  }

  /**
   * Parse and validate JSON response
   * @internal
   */
  private async parseJsonResponse<T>(response: Response): Promise<ApiResponse<T>> {
    const contentType = response.headers.get('content-type')
    const isJson = contentType?.includes('application/json')

    if (!isJson) {
      return {
        error: `Expected JSON response but received: ${contentType}`,
        status: response.status,
      }
    }

    const data = await response.json()
    return { data, status: response.status }
  }

  /**
   * Execute HTTP request with automatic error handling and token refresh
   * @param endpoint - API endpoint path (e.g., '/tasks')
   * @param options - Request configuration options
   * @returns Promise resolving to typed response with data or error
   *
   * @example
   * ```typescript
   * const result = await this.request<Task[]>('/tasks', { method: 'GET' })
   * ```
   */
  async request<T>(
    endpoint: string,
    options: RequestOptions = {}
  ): Promise<ApiResponse<T>> {
    const {
      method = 'GET',
      headers = {},
      body,
      authToken,
    } = options

    const url = `${this.baseUrl}${endpoint}`

    const fetchOptions: RequestInit = {
      method,
      headers: {
        'Content-Type': 'application/json',
        ...headers,
      },
      credentials: 'include',
    }

    if (authToken) {
      fetchOptions.headers = {
        ...fetchOptions.headers,
        Authorization: authToken,
      }
    }

    if (body && method !== 'GET') {
      fetchOptions.body = JSON.stringify(body)
    }

    try {
      const response = await fetch(url, fetchOptions)

      if (response.status === 429) {
        return { error: 'Rate limit exceeded. Please wait a moment and try again.', status: 429 }
      }

      if (response.status === 401) {
        return this.handle401Error(response, endpoint, options)
      }

      if (response.status === 403) {
        return { error: 'Forbidden. You do not have permission to access this resource.', status: 403 }
      }

      if (response.status === 204) {
        return { data: undefined as T, status: 204 }
      }

      if (!response.ok) {
        return this.handleErrorResponse(response)
      }

      return this.parseJsonResponse(response)
    } catch (error) {
      if (error instanceof TypeError && error.message.includes('fetch')) {
        return { error: 'Network error. Please check your connection and try again.' }
      }

      return {
        error: error instanceof Error ? error.message : 'Unknown error occurred',
      }
    }
  }

  /**
   * Fetch resource (GET)
   * @param endpoint - API endpoint path
   * @param authToken - Optional JWT authorization token
   * @returns Response containing data or error
   */
  async get<T>(endpoint: string, authToken?: string | null): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, { method: 'GET', authToken })
  }

  /**
   * Create resource (POST)
   * @param endpoint - API endpoint path
   * @param body - Request body data
   * @param authToken - Optional JWT authorization token
   * @returns Response containing created data or error
   */
  async post<T>(endpoint: string, body: any, authToken?: string | null): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, { method: 'POST', body, authToken })
  }

  /**
   * Update resource (PUT)
   * @param endpoint - API endpoint path
   * @param body - Request body data
   * @param authToken - Optional JWT authorization token
   * @returns Response containing updated data or error
   */
  async put<T>(endpoint: string, body: any, authToken?: string | null): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, { method: 'PUT', body, authToken })
  }

  /**
   * Delete resource (DELETE)
   * @param endpoint - API endpoint path
   * @param authToken - Optional JWT authorization token
   * @returns Response with delete status
   */
  async delete<T>(endpoint: string, authToken?: string | null): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, { method: 'DELETE', authToken })
  }

  /**
   * Partially update resource (PATCH)
   * @param endpoint - API endpoint path
   * @param body - Request body data with partial updates
   * @param authToken - Optional JWT authorization token
   * @returns Response containing updated data or error
   */
  async patch<T>(endpoint: string, body: any, authToken?: string | null): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, { method: 'PATCH', body, authToken })
  }
}

/** Singleton instance of API client */
export const apiClient = new ApiClient()

export { ApiClient }
export type { ApiResponse, RequestOptions }
