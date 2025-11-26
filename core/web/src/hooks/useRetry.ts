import { useState, useCallback } from 'react'
import { logger } from '@/lib/logger'

export interface UseRetryOptions {
  maxAttempts?: number
  initialDelay?: number
  backoff?: 'linear' | 'exponential'
  onAttempt?: (attempt: number, maxAttempts: number) => void
}

export interface UseRetryResult {
  retry: () => Promise<boolean>
  attempt: number
  isRetrying: boolean
  reset: () => void
  canRetry: boolean
}

/**
 * Hook for managing retry logic with exponential or linear backoff.
 *
 * Eliminates duplicate retry code across multiple hooks.
 * Provides type-safe retry management with automatic delay calculation.
 *
 * @param fn - Async function to retry
 * @param options - Retry configuration
 * @returns Retry state and methods
 *
 * @example
 * ```typescript
 * const { retry, attempt, isRetrying } = useRetry(
 *   connectToServer,
 *   { maxAttempts: 5, initialDelay: 2000, backoff: 'exponential' }
 * )
 *
 * // In handler
 * await retry()
 * ```
 */
export function useRetry(
  fn: () => Promise<void>,
  options: UseRetryOptions = {}
): UseRetryResult {
  const {
    maxAttempts = 5,
    initialDelay = 2000,
    backoff = 'exponential',
    onAttempt
  } = options

  const [attempt, setAttempt] = useState(0)
  const [isRetrying, setIsRetrying] = useState(false)

  const canRetry = attempt < maxAttempts

  /**
   * Executes the retry function with configured backoff delay.
   *
   * Waits for the calculated delay (based on backoff strategy), then attempts
   * to execute the function. Resets attempt counter on success, increments on failure.
   *
   * @returns Promise resolving to true if execution succeeds, false otherwise
   */
  const retry = useCallback(async (): Promise<boolean> => {
    if (!canRetry) {
      logger.error('Max retry attempts reached', undefined, 'useRetry')
      return false
    }

    setIsRetrying(true)
    const delay = backoff === 'exponential'
      ? initialDelay * Math.pow(2, attempt)
      : initialDelay * (attempt + 1)

    logger.debug('Retry', { attempt: attempt + 1, maxAttempts, delay }, 'useRetry')

    onAttempt?.(attempt + 1, maxAttempts)

    await new Promise(resolve => setTimeout(resolve, delay))

    try {
      await fn()
      setAttempt(0) // Reset on success
      setIsRetrying(false)
      logger.debug('Retry successful, reset attempts', undefined, 'useRetry')
      return true
    } catch (error) {
      setAttempt(prev => prev + 1)
      setIsRetrying(false)
      logger.debug('Retry failed', { attempt: attempt + 1, error }, 'useRetry')
      return false
    }
  }, [fn, attempt, maxAttempts, initialDelay, backoff, canRetry, onAttempt])

  /**
   * Resets the retry attempt counter and retrying state.
   *
   * Call this to allow retries again after max attempts were reached, or to
   * reset state after a successful operation.
   */
  const reset = useCallback(() => {
    setAttempt(0)
    setIsRetrying(false)
    logger.debug('Retry state reset', undefined, 'useRetry')
  }, [])

  return { retry, attempt, isRetrying, reset, canRetry }
}
