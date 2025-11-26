/**
 * HTTP status codes and headers used throughout the application.
 * Centralizes HTTP constants to eliminate magic numbers and string literals.
 * @module constants/http
 */

/**
 * Standard HTTP status codes used in API communication and error handling.
 * Using constants instead of magic numbers improves code clarity and maintainability.
 */
export const HTTP_STATUS = {
  OK: 200,
  CREATED: 201,
  ACCEPTED: 202,
  NO_CONTENT: 204,
  PARTIAL_CONTENT: 206,
  BAD_REQUEST: 400,
  UNAUTHORIZED: 401,
  FORBIDDEN: 403,
  NOT_FOUND: 404,
  CONFLICT: 409,
  UNPROCESSABLE_ENTITY: 422,
  RATE_LIMITED: 429,
  INTERNAL_ERROR: 500,
  SERVICE_UNAVAILABLE: 503,
} as const

/**
 * HTTP header names used in requests and responses.
 * Prevents typos in header names across the codebase.
 */
export const HTTP_HEADERS = {
  CONTENT_TYPE: 'Content-Type',
  AUTHORIZATION: 'Authorization',
  ACCEPT: 'Accept',
  CACHE_CONTROL: 'Cache-Control',
  X_TRACE_ID: 'X-Trace-ID',
  X_CONTEXT_ID: 'X-Context-ID',
} as const

/**
 * Common content type values for HTTP requests.
 * Standardizes content type declarations across the application.
 */
export const CONTENT_TYPES = {
  JSON: 'application/json',
  FORM_URLENCODED: 'application/x-www-form-urlencoded',
  EVENT_STREAM: 'text/event-stream',
  PLAIN_TEXT: 'text/plain',
} as const

/**
 * Authentication scheme constants for Authorization header.
 * Ensures consistent authentication header formatting.
 */
export const AUTH_SCHEME = {
  BEARER: 'Bearer',
  BASIC: 'Basic',
} as const
