/**
 * Async State Management Hook
 *
 * Simplifies managing async operation states (loading, error, data).
 * Replaces boilerplate useState + useCallback patterns.
 */

import { useState, useCallback } from 'react'
import { logger } from '@/lib/logger'

export type AsyncState<T> =
  | { status: 'idle'; data: null; error: null; loading: false }
  | { status: 'loading'; data: null; error: null; loading: true }
  | { status: 'success'; data: T; error: null; loading: false }
  | { status: 'error'; data: null; error: Error; loading: false }

export interface UseAsyncStateOptions {
  onSuccess?: (data: unknown) => void
  onError?: (error: Error) => void
  moduleId?: string
}

/**
 * Hook for managing async operation state.
 *
 * Handles loading, success, and error states automatically.
 * Reduces boilerplate compared to manual useState + useCallback.
 *
 * @param fn - Async function to execute
 * @param options - Configuration
 * @returns State and execute function
 *
 * @example
 * ```typescript
 * const { execute, status, data, error } = useAsyncState(
 *   async () => {
 *     const response = await fetch('/api/data')
 *     return response.json()
 *   }
 * )
 *
 * // In handler
 * await execute()
 * ```
 */
export function useAsyncState<T>(
  fn: () => Promise<T>,
  options: UseAsyncStateOptions = {}
) {
  const { onSuccess, onError, moduleId = 'useAsyncState' } = options

  const [state, setState] = useState<AsyncState<T>>({
    status: 'idle',
    data: null,
    error: null,
    loading: false
  })

  /**
   * Executes the async function and updates state accordingly.
   *
   * Sets loading state, executes the function, and transitions to success or
   * error state. Calls optional success/error callbacks and logs the result.
   *
   * @returns Promise resolving to the function result or null on error
   * @throws Error if execution fails (after updating error state)
   */
  const execute = useCallback(async (): Promise<T | null> => {
    setState({
      status: 'loading',
      data: null,
      error: null,
      loading: true
    })

    try {
      const result = await fn()

      setState({
        status: 'success',
        data: result,
        error: null,
        loading: false
      })

      onSuccess?.(result)
      logger.debug('Async operation succeeded', undefined, moduleId)

      return result
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err))

      setState({
        status: 'error',
        data: null,
        error,
        loading: false
      })

      onError?.(error)
      logger.error('Async operation failed', error, moduleId)

      throw error
    }
  }, [fn, onSuccess, onError, moduleId])

  /**
   * Resets async state to idle with no data or error.
   *
   * Use this to clear results after handling them or to prepare for a new execution.
   */
  const reset = useCallback(() => {
    setState({
      status: 'idle',
      data: null,
      error: null,
      loading: false
    })
    logger.debug('Async state reset', undefined, moduleId)
  }, [moduleId])

  return {
    ...state,
    execute,
    reset,
    isLoading: state.loading,
    isSuccess: state.status === 'success',
    isError: state.status === 'error',
  }
}
