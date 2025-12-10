/**
 * Server-Sent Events (SSE) Connection Management Hook
 *
 * Manages long-lived SSE connections with automatic reconnection, token refresh,
 * and exponential backoff. Handles all connection lifecycle events and errors.
 *
 * @module hooks/useSSEConnection
 *
 * @throws {Error} When connection fails after max reconnection attempts
 * @throws {Error} When authentication is invalid or missing
 *
 * @example
 * ```typescript
 * const { connect, disconnect } = useSSEConnection({
 *   url: '/api/stream',
 *   onMessage: (eventType, data) => console.log(eventType, data),
 *   onConnected: () => console.log('Connected'),
 *   onDisconnected: () => console.log('Disconnected')
 * })
 * ```
 */

import { useEffect, useRef, useCallback } from 'react'
import { useAuthStore } from '@/stores/auth.store'
import { useContextStore } from '@/stores/context.store'
import { useTaskStore } from '@/stores/task.store'
import { useSSEEventHandler } from './sse/useSSEEventHandler'
import { useSSETokenManagement } from './sse/useSSETokenManagement'
import { logger } from '@/lib/logger'

const MAX_RECONNECT_ATTEMPTS = 5
const RECONNECT_DELAY = 2000

/**
 * Configuration options for SSE connection
 */
export interface UseSSEConnectionOptions {
  /** URL endpoint for SSE stream */
  url: string
  /** Callback fired when an event is received with (eventType, data) */
  onMessage?: (eventType: string, data: string) => void
  /** Callback fired when connection is established */
  onConnected?: () => void
  /** Callback fired when connection is closed or disconnected */
  onDisconnected?: () => void
  /** Callback fired when a connection error occurs */
  onError?: (error: Error) => void
}

/**
 * Result of useSSEConnection hook
 */
export interface UseSSEConnectionResult {
  /** Initiate connection to SSE endpoint */
  connect: () => void
  /** Close connection and stop attempting to reconnect */
  disconnect: () => void
  /** Current connection status */
  isConnected: boolean
  /** Last error encountered, null if no error */
  error: Error | null
}

/**
 * Hook for managing SSE connections with automatic reconnection.
 *
 * Automatically connects when authentication is available and disconnects when
 * authentication is lost. Handles token refresh on 401 errors and implements
 * exponential backoff for reconnection attempts.
 *
 * Delegates to specialized hooks:
 * - useSSEEventHandler: Stream event parsing
 * - useSSETokenManagement: Token refresh logic
 *
 * @param options - Configuration options for the connection
 * @returns Hook result with connect/disconnect methods and connection state
 *
 * @example
 * ```typescript
 * const { connect, disconnect, isConnected } = useSSEConnection({
 *   url: '/api/live-data',
 *   onMessage: (type, data) => console.log(type, data),
 *   onConnected: () => console.log('Connected to live data'),
 * })
 * ```
 */
export function useSSEConnection(options: UseSSEConnectionOptions): UseSSEConnectionResult {
  const { url, onMessage, onConnected, onDisconnected } = options

  const abortControllerRef = useRef<AbortController | null>(null)
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null)
  const reconnectAttemptsRef = useRef(0)
  const isConnectedRef = useRef(false)
  const errorRef = useRef<Error | null>(null)
  const hasConnectedBeforeRef = useRef(false)

  const { accessToken, isAuthenticated, userId } = useAuthStore()
  const { processSSEStream } = useSSEEventHandler(onMessage)
  const { handleTokenRefresh } = useSSETokenManagement(onDisconnected)

  const scheduleReconnect = useCallback((connect: () => void) => {
    if (reconnectAttemptsRef.current >= MAX_RECONNECT_ATTEMPTS) {
      logger.error('Max reconnect attempts reached', undefined, 'useSSEConnection')
      useContextStore.getState().setSSEStatus('disconnected')
      onDisconnected?.()
      return
    }

    const delay = RECONNECT_DELAY * Math.pow(2, reconnectAttemptsRef.current)
    logger.info('Reconnecting', { delay, attempt: reconnectAttemptsRef.current + 1, max: MAX_RECONNECT_ATTEMPTS }, 'useSSEConnection')

    reconnectTimeoutRef.current = setTimeout(() => {
      reconnectAttemptsRef.current++
      connect()
    }, delay)
  }, [onDisconnected])

  const handleConnectionError = useCallback(async (error?: unknown, connect?: () => void) => {
    useContextStore.getState().setSSEStatus('error')
    useContextStore.getState().setSSEError('Connection failed')
    errorRef.current = error instanceof Error ? error : new Error(String(error))
    abortControllerRef.current = null

    const errorMessage = error instanceof Error ? error.message : String(error)
    const errorStatus = (error as { status?: number } | undefined)?.status
    const is401Error = errorMessage.includes('401') || errorStatus === 401
    const isAnonUser = useAuthStore.getState().userType === 'anon'

    if (is401Error && isAnonUser) {
      logger.info('401 Unauthorized - attempting token refresh', undefined, 'useSSEConnection')
      const tokenRefreshed = await handleTokenRefresh()
      if (tokenRefreshed && connect) {
        reconnectAttemptsRef.current = 0
        connect()
        return
      }
    }

    if (connect) {
      scheduleReconnect(connect)
    }
  }, [handleTokenRefresh, scheduleReconnect])

  const isAuthValid = (): boolean => {
    const tokenValid = useAuthStore.getState().isTokenValid()
    if (!isAuthenticated || !userId || !accessToken || !tokenValid) {
      logger.debug('Cannot connect - auth validation failed', { isAuthenticated, userId: !!userId, tokenValid }, 'useSSEConnection')
      return false
    }
    return true
  }

  /**
   * Recover UI state after SSE reconnection.
   * This ensures messages and task data are restored if they were lost during disconnection.
   */
  const recoverStateAfterReconnection = useCallback(async () => {
    const contextId = useContextStore.getState().currentContextId
    if (!contextId) {
      logger.debug('No current context to recover', undefined, 'useSSEConnection')
      return
    }

    logger.info('Recovering state after reconnection', { contextId }, 'useSSEConnection')

    try {
      const authHeader = useAuthStore.getState().getAuthHeader()
      await useTaskStore.getState().fetchTasksByContext(contextId, authHeader)
      logger.info('State recovery completed', { contextId }, 'useSSEConnection')
    } catch (error) {
      logger.error('Failed to recover state after reconnection', error, 'useSSEConnection')
    }
  }, [])

  const connect = useCallback(() => {
    if (!isAuthValid()) return
    if (abortControllerRef.current) {
      logger.debug('Already connected', undefined, 'useSSEConnection')
      return
    }

    logger.info('Connecting to SSE', undefined, 'useSSEConnection')
    useContextStore.getState().setSSEStatus('connecting')

    const abortController = new AbortController()
    abortControllerRef.current = abortController

    const runStream = async () => {
      try {
        const currentAuthHeader = useAuthStore.getState().getAuthHeader()
        if (!currentAuthHeader) {
          logger.error('Missing auth header', undefined, 'useSSEConnection')
          throw new Error('Missing authentication')
        }

        const response = await fetch(url, {
          method: 'GET',
          headers: {
            'Authorization': currentAuthHeader,
            'Accept': 'text/event-stream',
            'Cache-Control': 'no-cache',
          },
          signal: abortController.signal,
        })

        if (!response.ok) {
          throw new Error(`SSE request failed: ${response.status} ${response.statusText}`)
        }

        if (!response.body) {
          throw new Error('Response body is null')
        }

        logger.info('Connected to SSE', undefined, 'useSSEConnection')
        useContextStore.getState().setSSEStatus('connected')
        useContextStore.getState().setSSEError(null)

        const isReconnection = hasConnectedBeforeRef.current
        hasConnectedBeforeRef.current = true
        reconnectAttemptsRef.current = 0
        isConnectedRef.current = true
        errorRef.current = null
        onConnected?.()

        // Recover state after reconnection to ensure UI is in sync with backend
        if (isReconnection) {
          recoverStateAfterReconnection()
        }

        await processSSEStream(response.body.getReader())
      } catch (error: unknown) {
        if (error instanceof Error && error.name === 'AbortError') {
          logger.debug('Connection aborted', undefined, 'useSSEConnection')
          isConnectedRef.current = false
          onDisconnected?.()
          return
        }

        logger.error('SSE connection error', error, 'useSSEConnection')
        isConnectedRef.current = false
        handleConnectionError(error, connect)
      } finally {
        abortControllerRef.current = null
      }
    }

    runStream()
  }, [isAuthenticated, userId, accessToken, url, onConnected, onDisconnected, handleConnectionError, processSSEStream, recoverStateAfterReconnection])

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current)
      reconnectTimeoutRef.current = null
    }

    if (abortControllerRef.current) {
      logger.debug('Disconnecting from SSE', undefined, 'useSSEConnection')
      abortControllerRef.current.abort()
      abortControllerRef.current = null
      isConnectedRef.current = false
      useContextStore.getState().setSSEStatus('disconnected')
      onDisconnected?.()
    }
  }, [onDisconnected])

  useEffect(() => {
    const tokenValid = useAuthStore.getState().isTokenValid()
    const hasAuthHeader = !!useAuthStore.getState().getAuthHeader()

    if (isAuthenticated && userId && accessToken && tokenValid && hasAuthHeader) {
      connect()
    }

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current)
        reconnectTimeoutRef.current = null
      }
      if (abortControllerRef.current) {
        abortControllerRef.current.abort()
        abortControllerRef.current = null
      }
    }
  }, [isAuthenticated, userId, accessToken, connect])

  return {
    connect,
    disconnect,
    isConnected: isConnectedRef.current,
    error: errorRef.current,
  }
}
