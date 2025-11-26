/**
 * JWT Utility Functions
 *
 * Type-safe utilities for decoding and extracting claims from JWT tokens.
 * Handles JWT format validation, Base64URL decoding, and claim extraction.
 *
 * Supports standard and custom claims:
 * - Standard: sub, email, exp, iat, scope
 * - Custom: session_id, user_type, username
 *
 * @example
 * ```typescript
 * import {
 *   extractUserIdFromJWT,
 *   extractSessionIdFromJWT,
 *   extractScopesFromJWT
 * } from '@/utils/jwt'
 *
 * const userId = extractUserIdFromJWT(token)
 * const sessionId = extractSessionIdFromJWT(token)
 * const scopes = extractScopesFromJWT(token)
 * ```
 *
 * @throws {Error} If JWT is malformed or required claims are missing
 */

/**
 * JWT claims payload extracted from token
 * Includes both standard IANA claims and application-specific claims
 *
 * @typedef {Object} JWTClaims
 * @property {string} [sub] - Subject claim (user ID)
 * @property {string} [email] - Email address
 * @property {string} [session_id] - Session identifier
 * @property {string} [scope] - Space-separated OAuth scopes
 * @property {string} [user_type] - User type (e.g., 'anon', 'authenticated')
 * @property {string} [username] - Display name or username
 * @property {number} [exp] - Expiration time (Unix timestamp)
 * @property {number} [iat] - Issued at time (Unix timestamp)
 * @property {Record<string, unknown>} [key] - Additional custom claims
 */
interface JWTClaims {
  sub?: string
  email?: string
  session_id?: string
  scope?: string
  user_type?: string
  username?: string
  exp?: number
  iat?: number
  [key: string]: unknown
}

/**
 * Decode JWT token and extract claims payload
 *
 * Parses the JWT structure (header.payload.signature), validates format,
 * decodes the Base64URL-encoded payload, and returns parsed claims.
 *
 * Process:
 * 1. Split token by '.' separator (must have exactly 3 parts)
 * 2. Extract and decode Base64URL payload (part [1])
 * 3. Parse JSON claims object
 *
 * @param {string} token - JWT token string to decode
 * @returns {JWTClaims} Decoded claims object
 * @throws {Error} If token format is invalid or payload cannot be parsed
 *
 * @example
 * ```typescript
 * const token = "eyJhbGc..." // Valid JWT
 * const claims = decodeJWT(token)
 * console.log(claims.sub) // User ID
 * ```
 */
export function decodeJWT(token: string): JWTClaims {
  try {
    const parts = token.split('.')
    if (parts.length !== 3) {
      throw new Error('Invalid JWT format: expected 3 parts separated by dots')
    }

    const payload = parts[1]
    const decoded = atob(payload.replace(/-/g, '+').replace(/_/g, '/'))
    const claims = JSON.parse(decoded) as JWTClaims

    return claims
  } catch (error) {
    throw new Error(
      `Failed to decode JWT: ${error instanceof Error ? error.message : 'Unknown error'}`
    )
  }
}

/**
 * Extract user ID from JWT token
 *
 * Decodes token and returns the 'sub' (subject) claim which represents
 * the authenticated user's unique identifier.
 *
 * @param {string} token - JWT token to extract from
 * @returns {string} User ID from 'sub' claim
 * @throws {Error} If token is malformed or 'sub' claim is missing
 *
 * @example
 * ```typescript
 * const userId = extractUserIdFromJWT(token)
 * // Returns: "user-uuid-here"
 * ```
 */
export function extractUserIdFromJWT(token: string): string {
  const claims = decodeJWT(token)
  if (!claims.sub) {
    throw new Error('JWT missing required "sub" claim (user ID)')
  }
  return claims.sub
}

/**
 * Extract session ID from JWT token
 *
 * Retrieves the application-specific 'session_id' claim which tracks
 * the user's current session for context correlation and logging.
 *
 * @param {string} token - JWT token to extract from
 * @returns {string} Session ID from 'session_id' claim
 * @throws {Error} If token is malformed or 'session_id' claim is missing
 *
 * @example
 * ```typescript
 * const sessionId = extractSessionIdFromJWT(token)
 * // Returns: "session-uuid-here"
 * ```
 */
export function extractSessionIdFromJWT(token: string): string {
  const claims = decodeJWT(token)
  if (!claims.session_id) {
    throw new Error('JWT missing required "session_id" claim')
  }
  return claims.session_id
}

/**
 * Extract email address from JWT token
 *
 * Retrieves the email claim for user identification and communication.
 * May not be available for all user types (e.g., anonymous users).
 *
 * @param {string} token - JWT token to extract from
 * @returns {string} Email address from 'email' claim
 * @throws {Error} If token is malformed or 'email' claim is missing
 *
 * @example
 * ```typescript
 * const email = extractEmailFromJWT(token)
 * // Returns: "user@example.com"
 * ```
 */
export function extractEmailFromJWT(token: string): string {
  const claims = decodeJWT(token)
  if (!claims.email) {
    throw new Error('JWT missing required "email" claim')
  }
  return claims.email
}

/**
 * Extract OAuth 2.0 scopes from JWT token
 *
 * Parses the space-separated 'scope' claim and returns as array.
 * Returns empty array if scope claim is absent or empty.
 *
 * Scopes define authorization level and API access:
 * - 'user': Standard authenticated user permissions
 * - 'admin': Administrator permissions
 *
 * @param {string} token - JWT token to extract from
 * @returns {string[]} Array of scope strings (empty if none)
 *
 * @example
 * ```typescript
 * const scopes = extractScopesFromJWT(token)
 * // Returns: ["user", "admin"]
 *
 * if (scopes.includes('admin')) {
 *   showAdminPanel()
 * }
 * ```
 */
export function extractScopesFromJWT(token: string): string[] {
  const claims = decodeJWT(token)
  return claims.scope ? claims.scope.split(' ').filter((s) => s.length > 0) : []
}

/**
 * Extract user type from JWT token
 *
 * Returns the 'user_type' claim indicating authentication method or user category.
 * Common values: 'anon' (anonymous), 'authenticated', 'service'.
 *
 * @param {string} token - JWT token to extract from
 * @returns {string | null} User type from 'user_type' claim or null if absent
 *
 * @example
 * ```typescript
 * const userType = extractUserTypeFromJWT(token)
 * if (userType === 'anon') {
 *   promptLoginDialog()
 * }
 * ```
 */
export function extractUserTypeFromJWT(token: string): string | null {
  const claims = decodeJWT(token)
  return claims.user_type || null
}

/**
 * Extract username from JWT token
 *
 * Returns the 'username' claim for display purposes.
 * May differ from email and is typically the user's chosen display name.
 *
 * @param {string} token - JWT token to extract from
 * @returns {string | null} Username from 'username' claim or null if absent
 *
 * @example
 * ```typescript
 * const username = extractUsernameFromJWT(token)
 * console.log(`Welcome, ${username}!`)
 * ```
 */
export function extractUsernameFromJWT(token: string): string | null {
  const claims = decodeJWT(token)
  return claims.username || null
}
