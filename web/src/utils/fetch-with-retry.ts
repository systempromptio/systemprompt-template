/**
 * Fetch with Automatic Retry Utility
 *
 * Wraps fetch requests with automatic retry logic, exponential backoff,
 * and jitter to handle transient failures gracefully.
 *
 * @example
 * ```typescript
 * const data = await fetchWithRetry<Task[]>(
 *   () => fetch('/api/tasks'),
 *   { maxRetries: 5, baseDelay: 1000 }
 * )
 * ```
 */

import { logger } from '@/lib/logger'

interface ErrorWithResponse {
  response?: {
    status?: number
    headers?: Record<string, string>
  }
}

function hasResponseStatus(error: unknown): error is ErrorWithResponse {
  if (typeof error !== 'object' || error === null) return false
  const err = error as Record<string, unknown>
  if (!err.response || typeof err.response !== 'object') return false
  return true
}

function getResponseStatus(error: unknown): number | undefined {
  if (!hasResponseStatus(error)) return undefined
  return error.response?.status
}

/**
 * Configuration options for fetch retry behavior
 */
interface RetryOptions {
  /** Maximum number of retry attempts (default: 3) */
  maxRetries?: number
  /** Initial delay in milliseconds (default: 1000) */
  baseDelay?: number
  /** Maximum delay in milliseconds (default: 10000) */
  maxDelay?: number
  /** Function to determine if error is retryable (default: retry on 429, 5xx) */
  shouldRetry?: (error: unknown) => boolean
}

const defaultOptions: Required<RetryOptions> = {
  maxRetries: 3,
  baseDelay: 1000,
  maxDelay: 10000,
  shouldRetry: (error: unknown) => {
    const status = getResponseStatus(error)
    if (status === 429) return true
    if (status !== undefined && status >= 500) return true
    return false
  },
}

/**
 * Execute function with exponential backoff retry strategy
 *
 * Retries on errors matching shouldRetry predicate with exponential backoff
 * and random jitter. Failures after maxRetries are re-thrown.
 *
 * @param fn - Async function to execute with retry
 * @param options - Retry configuration options
 * @returns Result of successful fn() call
 * @throws Error if fn fails after maxRetries attempts
 *
 * @example
 * ```typescript
 * const response = await fetchWithRetry(
 *   () => fetch('/api/data'),
 *   { maxRetries: 5, baseDelay: 500 }
 * )
 * ```
 */
export async function fetchWithRetry<T>(
  fn: () => Promise<T>,
  options: RetryOptions = {}
): Promise<T> {
  const opts = { ...defaultOptions, ...options }

  for (let attempt = 0; attempt < opts.maxRetries; attempt++) {
    try {
      return await fn()
    } catch (error: unknown) {
      const isLastAttempt = attempt === opts.maxRetries - 1

      if (isLastAttempt || !opts.shouldRetry(error)) {
        throw error
      }

      const retryAfterMs = getRetryAfter(error)
      let delay: number

      if (retryAfterMs !== null) {
        delay = retryAfterMs
      } else {
        delay = Math.min(opts.baseDelay * Math.pow(2, attempt), opts.maxDelay)
      }

      const jitter = Math.random() * 200
      const totalDelay = delay + jitter

      logger.debug(
        `Retry attempt ${attempt + 1}/${opts.maxRetries}`,
        { delayMs: Math.round(totalDelay), status: getResponseStatus(error) },
        'fetch-with-retry'
      )

      await new Promise((resolve) => setTimeout(resolve, totalDelay))
    }
  }

  throw new Error('Max retries exceeded')
}

/**
 * Detect if error is a rate limit (HTTP 429) response
 *
 * Identifies when the server has rate-limited the client, signaling
 * too many requests in a given period. Useful for triggering backoff
 * strategies and alerting users to retry later.
 *
 * @param {any} error - Error object from failed request
 * @returns {boolean} True if error response status is 429
 *
 * @example
 * ```typescript
 * try {
 *   await fetchData()
 * } catch (error) {
 *   if (isRateLimitError(error)) {
 *     showRateLimitMessage()
 *   }
 * }
 * ```
 */
export function isRateLimitError(error: unknown): boolean {
  return getResponseStatus(error) === 429
}

/**
 * Extract Retry-After delay from HTTP error response
 *
 * Parses the standard HTTP 'Retry-After' header which instructs
 * the client when to retry after rate limiting or temporary unavailability.
 * Converts server-provided delay in seconds to milliseconds.
 *
 * Returns null if:
 * - Header is missing or empty
 * - Value cannot be parsed as integer
 * - Error lacks response structure
 *
 * @param {any} error - Error object from failed request
 * @returns {number | null} Delay in milliseconds or null if unavailable
 *
 * @example
 * ```typescript
 * try {
 *   await fetch(url)
 * } catch (error) {
 *   const delay = getRetryAfter(error) ?? 5000
 *   await sleep(delay)
 *   retry()
 * }
 * ```
 */
export function getRetryAfter(error: unknown): number | null {
  if (!hasResponseStatus(error)) return null
  const headers = error.response?.headers
  if (!headers) return null
  const retryAfter = headers['retry-after']
  if (retryAfter) {
    const seconds = parseInt(retryAfter, 10)
    return isNaN(seconds) ? null : seconds * 1000
  }
  return null
}
