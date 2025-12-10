/**
 * API Response Validators
 *
 * Validates API responses at the boundary before they enter the application.
 * Ensures runtime type safety for data coming from external APIs.
 *
 * Usage:
 * const response = await apiClient.get('/tasks')
 * const tasks = validateArrayResponse(response, isTask, 'GET /tasks')
 */

import { isPlainObject } from '@/utils/type-guards'

/**
 * Custom error for API validation failures.
 * Captures the context and raw data for debugging.
 */
export class ApiValidationError extends Error {
  context: string
  rawData: unknown

  constructor(context: string, rawData: unknown) {
    super(`API validation failed: ${context}`)
    this.name = 'ApiValidationError'
    this.context = context
    this.rawData = rawData
  }
}

/**
 * Validate API response has expected shape.
 * Throws ApiValidationError if validation fails.
 *
 * @param response - The raw API response
 * @param validator - Type guard function to validate the response
 * @param context - Description of the API call for error messages
 * @returns The validated response with correct type
 */
export function validateApiResponse<T>(
  response: unknown,
  validator: (data: unknown) => data is T,
  context: string
): T {
  if (!validator(response)) {
    throw new ApiValidationError(context, response)
  }
  return response
}

/**
 * Validate array response where all items must pass validation.
 * Throws ApiValidationError if response is not an array or any item fails validation.
 *
 * @param response - The raw API response
 * @param itemValidator - Type guard function to validate each item
 * @param context - Description of the API call for error messages
 * @returns The validated array with correct item types
 */
export function validateArrayResponse<T>(
  response: unknown,
  itemValidator: (item: unknown) => item is T,
  context: string
): T[] {
  if (!Array.isArray(response)) {
    throw new ApiValidationError(`${context}: expected array, got ${typeof response}`, response)
  }

  const validated: T[] = []
  for (let i = 0; i < response.length; i++) {
    if (!itemValidator(response[i])) {
      throw new ApiValidationError(`${context}[${i}]: invalid item`, response[i])
    }
    validated.push(response[i])
  }

  return validated
}

/**
 * Validate array response with lenient handling - skip invalid items instead of throwing.
 * Useful when partial data is acceptable.
 *
 * @param response - The raw API response
 * @param itemValidator - Type guard function to validate each item
 * @param context - Description of the API call for logging
 * @param logger - Optional logger for invalid item warnings
 * @returns Array of valid items only
 */
export function validateArrayResponseLenient<T>(
  response: unknown,
  itemValidator: (item: unknown) => item is T,
  context: string,
  logger?: { warn: (msg: string, data: unknown, source: string) => void }
): T[] {
  if (!Array.isArray(response)) {
    logger?.warn(`${context}: expected array, got ${typeof response}`, response, 'validators')
    return []
  }

  const validated: T[] = []
  for (let i = 0; i < response.length; i++) {
    if (itemValidator(response[i])) {
      validated.push(response[i])
    } else {
      logger?.warn(`${context}[${i}]: invalid item, skipping`, response[i], 'validators')
    }
  }

  return validated
}

/**
 * Validate optional API response - returns undefined if response is null/undefined.
 *
 * @param response - The raw API response
 * @param validator - Type guard function to validate the response
 * @param context - Description of the API call for error messages
 * @returns The validated response or undefined
 */
export function validateOptionalApiResponse<T>(
  response: unknown,
  validator: (data: unknown) => data is T,
  context: string
): T | undefined {
  if (response === null || response === undefined) {
    return undefined
  }
  return validateApiResponse(response, validator, context)
}

/**
 * Validate paginated response with items array.
 *
 * @param response - The raw API response
 * @param itemValidator - Type guard function to validate each item
 * @param context - Description of the API call for error messages
 * @returns Object with validated items and pagination info
 */
export function validatePaginatedResponse<T>(
  response: unknown,
  itemValidator: (item: unknown) => item is T,
  context: string
): { items: T[]; total?: number; page?: number; pageSize?: number } {
  if (!isPlainObject(response)) {
    throw new ApiValidationError(`${context}: expected object`, response)
  }

  const items = 'items' in response ? response.items : 'data' in response ? response.data : undefined

  if (items === undefined) {
    throw new ApiValidationError(`${context}: missing items/data field`, response)
  }

  const validatedItems = validateArrayResponse(items, itemValidator, `${context}.items`)

  return {
    items: validatedItems,
    total: typeof response.total === 'number' ? response.total : undefined,
    page: typeof response.page === 'number' ? response.page : undefined,
    pageSize: typeof response.pageSize === 'number' ? response.pageSize : undefined,
  }
}

/**
 * Create a validator that checks for specific required fields.
 * Useful for ad-hoc validation without defining a full type guard.
 *
 * @param requiredFields - Array of field names that must be present
 * @returns Type guard function
 */
export function createFieldValidator<T extends Record<string, unknown>>(
  requiredFields: (keyof T)[]
): (data: unknown) => data is T {
  return (data: unknown): data is T => {
    if (!isPlainObject(data)) return false
    return requiredFields.every((field) => field in data)
  }
}

/**
 * Wrap an async API call with validation.
 * Catches errors and provides consistent error handling.
 *
 * @param apiCall - The async API call to wrap
 * @param validator - Type guard function to validate the response
 * @param context - Description of the API call for error messages
 * @returns The validated response
 */
export async function withValidation<T>(
  apiCall: () => Promise<unknown>,
  validator: (data: unknown) => data is T,
  context: string
): Promise<T> {
  const response = await apiCall()
  return validateApiResponse(response, validator, context)
}

/**
 * Wrap an async API call with array validation.
 *
 * @param apiCall - The async API call to wrap
 * @param itemValidator - Type guard function to validate each item
 * @param context - Description of the API call for error messages
 * @returns The validated array
 */
export async function withArrayValidation<T>(
  apiCall: () => Promise<unknown>,
  itemValidator: (item: unknown) => item is T,
  context: string
): Promise<T[]> {
  const response = await apiCall()
  return validateArrayResponse(response, itemValidator, context)
}
