/**
 * Centralized retry logic and strategies for async operations.
 *
 * Provides reusable retry mechanisms with exponential backoff, jitter,
 * and configurable error handling across the application.
 *
 * @module utils/retry-strategy
 */

/**
 * Configuration for retry behavior.
 */
export interface RetryConfig {
  /**
   * Maximum number of retry attempts (not including initial attempt)
   * @default 3
   */
  maxRetries: number

  /**
   * Initial delay in milliseconds before first retry
   * @default 1000
   */
  initialDelayMs: number

  /**
   * Maximum delay in milliseconds between retries
   * @default 5000
   */
  maxDelayMs: number

  /**
   * Multiplier for exponential backoff
   * @default 2
   */
  backoffMultiplier: number

  /**
   * Whether to add random jitter to delays
   * @default true
   */
  useJitter: boolean

  /**
   * Optional function to determine if error is retryable
   * @param error The error to evaluate
   * @returns true if the operation should be retried
   */
  isRetryable?: (error: unknown) => boolean
}

/**
 * Result of a retry attempt.
 */
export interface RetryResult<T> {
  /**
   * Success value if operation succeeded
   */
  value?: T

  /**
   * Error if operation failed
   */
  error?: unknown

  /**
   * Number of attempts made
   */
  attempts: number

  /**
   * Whether operation ultimately succeeded
   */
  success: boolean
}

/**
 * Default retry configuration.
 *
 * @constant
 */
export const DEFAULT_RETRY_CONFIG: RetryConfig = {
  maxRetries: 3,
  initialDelayMs: 1000,
  maxDelayMs: 5000,
  backoffMultiplier: 2,
  useJitter: true,
}

/**
 * Sleep for a specified duration.
 *
 * @param ms Duration in milliseconds
 * @returns Promise that resolves after the delay
 *
 * @internal
 */
function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

/**
 * Calculate delay with exponential backoff and optional jitter.
 *
 * @param attempt Current attempt number (0-indexed)
 * @param config Retry configuration
 * @returns Delay in milliseconds
 *
 * @internal
 */
function calculateDelay(attempt: number, config: RetryConfig): number {
  const exponentialDelay = config.initialDelayMs * Math.pow(config.backoffMultiplier, attempt)
  const cappedDelay = Math.min(exponentialDelay, config.maxDelayMs)

  if (!config.useJitter) {
    return cappedDelay
  }

  const jitter = Math.random() * cappedDelay * 0.1
  return cappedDelay + jitter
}

/**
 * Default error evaluation: retry on network errors.
 *
 * @param error The error to evaluate
 * @returns true if error appears to be a network error
 *
 * @internal
 */
function isNetworkError(error: unknown): boolean {
  if (error instanceof Error) {
    const message = error.message.toLowerCase()
    return (
      message.includes('network') ||
      message.includes('timeout') ||
      message.includes('econnrefused') ||
      message.includes('enotfound') ||
      message.includes('abort')
    )
  }
  return false
}

/**
 * Retry an async operation with exponential backoff.
 *
 * Executes the provided function, automatically retrying on failure with
 * exponential backoff and optional jitter. Respects the configured retry
 * policy and allows custom retry predicate.
 *
 * @template T The return type of the operation
 * @param fn Async function to retry
 * @param config Optional retry configuration
 * @returns Result with value, error, and attempt count
 *
 * @example
 * ```typescript
 * const result = await withRetry(
 *   () => fetch(url),
 *   { maxRetries: 3, maxDelayMs: 3000 }
 * )
 *
 * if (result.success) {
 *   console.log('Success after', result.attempts, 'attempts')
 * } else {
 *   console.error('Failed:', result.error)
 * }
 * ```
 */
export async function withRetry<T>(
  fn: () => Promise<T>,
  config: Partial<RetryConfig> = {}
): Promise<RetryResult<T>> {
  const finalConfig: RetryConfig = { ...DEFAULT_RETRY_CONFIG, ...config }
  const isRetryable = config.isRetryable || isNetworkError

  let lastError: unknown

  for (let attempt = 0; attempt <= finalConfig.maxRetries; attempt++) {
    try {
      const value = await fn()
      return {
        value,
        attempts: attempt + 1,
        success: true,
      }
    } catch (error) {
      lastError = error

      if (attempt >= finalConfig.maxRetries) {
        break
      }

      if (!isRetryable(error)) {
        return {
          error,
          attempts: attempt + 1,
          success: false,
        }
      }

      const delay = calculateDelay(attempt, finalConfig)
      await sleep(delay)
    }
  }

  return {
    error: lastError,
    attempts: finalConfig.maxRetries + 1,
    success: false,
  }
}

/**
 * Retry an async operation with custom callback on each attempt.
 *
 * Similar to withRetry but allows custom handling of each attempt,
 * useful for logging, UI updates, or state management.
 *
 * @template T The return type of the operation
 * @param fn Async function to retry
 * @param onAttempt Callback invoked after each attempt
 * @param config Optional retry configuration
 * @returns Result with value, error, and attempt count
 *
 * @example
 * ```typescript
 * const result = await withRetryCallback(
 *   () => fetchData(url),
 *   (attempt, error) => {
 *     if (error) {
 *       console.log(`Attempt ${attempt} failed:`, error.message)
 *     }
 *   },
 *   { maxRetries: 5 }
 * )
 * ```
 */
export async function withRetryCallback<T>(
  fn: () => Promise<T>,
  onAttempt: (attempt: number, error?: unknown) => void,
  config: Partial<RetryConfig> = {}
): Promise<RetryResult<T>> {
  const finalConfig: RetryConfig = { ...DEFAULT_RETRY_CONFIG, ...config }
  const isRetryable = config.isRetryable || isNetworkError

  let lastError: unknown

  for (let attempt = 0; attempt <= finalConfig.maxRetries; attempt++) {
    try {
      const value = await fn()
      if (attempt > 0) {
        onAttempt(attempt + 1)
      }
      return {
        value,
        attempts: attempt + 1,
        success: true,
      }
    } catch (error) {
      lastError = error
      onAttempt(attempt + 1, error)

      if (attempt >= finalConfig.maxRetries) {
        break
      }

      if (!isRetryable(error)) {
        return {
          error,
          attempts: attempt + 1,
          success: false,
        }
      }

      const delay = calculateDelay(attempt, finalConfig)
      await sleep(delay)
    }
  }

  return {
    error: lastError,
    attempts: finalConfig.maxRetries + 1,
    success: false,
  }
}

/**
 * Create a reusable retry strategy with fixed configuration.
 *
 * Useful for creating custom retry behavior for specific operation types
 * with consistent configuration across the application.
 *
 * @param config Retry configuration
 * @returns Function that retries operations with the fixed config
 *
 * @example
 * ```typescript
 * const apiRetry = createRetryStrategy({
 *   maxRetries: 5,
 *   maxDelayMs: 10000,
 *   isRetryable: (error) => error.status >= 500
 * })
 *
 * const result = await apiRetry(() => fetch(url))
 * ```
 */
export function createRetryStrategy(config: Partial<RetryConfig>) {
  return <T>(fn: () => Promise<T>) => withRetry(fn, config)
}

/**
 * Pre-configured retry strategy for network requests.
 *
 * Optimized for HTTP requests with longer delays for server errors.
 *
 * @constant
 */
export const networkRetry = createRetryStrategy({
  maxRetries: 3,
  initialDelayMs: 500,
  maxDelayMs: 5000,
  backoffMultiplier: 2,
  useJitter: true,
})

/**
 * Pre-configured retry strategy for database operations.
 *
 * Optimized for quick retries on transient DB errors.
 *
 * @constant
 */
export const databaseRetry = createRetryStrategy({
  maxRetries: 2,
  initialDelayMs: 100,
  maxDelayMs: 1000,
  backoffMultiplier: 2,
  useJitter: false,
})

/**
 * Pre-configured retry strategy for agent communication.
 *
 * Optimized for A2A client connections with aggressive retry.
 *
 * @constant
 */
export const agentRetry = createRetryStrategy({
  maxRetries: 5,
  initialDelayMs: 1000,
  maxDelayMs: 10000,
  backoffMultiplier: 2,
  useJitter: true,
})
