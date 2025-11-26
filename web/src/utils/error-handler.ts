/**
 * Error normalization utilities.
 * Provides consistent error handling across the application.
 * @module utils/error-handler
 */

/**
 * Normalize any error type to Error instance with custom message.
 * Handles common error patterns: Error objects, strings, and unknown types.
 * Ensures all error handling code works with consistent Error type.
 * @param error - Error in any form (Error object, string, or unknown type)
 * @param defaultMessage - Fallback message if error type cannot be determined
 * @returns Normalized Error instance
 * @example
 * ```typescript
 * try {
 *   await someOperation()
 * } catch (err) {
 *   const error = normalizeError(err, 'Operation failed')
 *   console.error(error.message)
 * }
 * ```
 */
export function normalizeError(error: unknown, defaultMessage = 'An error occurred'): Error {
  if (error instanceof Error) return error
  if (typeof error === 'string') return new Error(error)
  return new Error(defaultMessage)
}

/**
 * Extract user-friendly error message from any error source.
 * Attempts to extract the most relevant error message from various error formats.
 * @param error - Error in any form
 * @param defaultMessage - Fallback message
 * @returns User-friendly error message string
 */
export function getErrorMessage(error: unknown, defaultMessage = 'An error occurred'): string {
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  if (error && typeof error === 'object' && 'message' in error) {
    return String((error as Record<string, unknown>).message)
  }
  return defaultMessage
}
