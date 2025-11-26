/**
 * Consistent date and time parsing utilities.
 *
 * Provides centralized date parsing and formatting to ensure consistency
 * across the application and handle multiple datetime formats.
 *
 * @module utils/date-parsing
 */

/**
 * Parse datetime string in multiple formats to Date object.
 *
 * Handles:
 * - RFC3339 format: "2024-01-01T12:00:00Z" or "2024-01-01T12:00:00+00:00"
 * - SQLite format: "2024-01-01 12:00:00" or "2024-01-01 12:00:00.000"
 * - ISO format: "2024-01-01T12:00:00"
 * - Unix timestamps (as number or string)
 *
 * @param value - Datetime string or timestamp to parse
 * @returns Parsed Date object, or null if parsing fails
 *
 * @throws Never throws; returns null on error
 *
 * @example
 * ```typescript
 * parseDateTime('2024-01-01T12:00:00Z') // Date object
 * parseDateTime('2024-01-01 12:00:00') // Date object
 * parseDateTime(1704110400000) // Date object
 * parseDateTime('invalid') // null
 * ```
 */
export function parseDateTime(value: string | number | null | undefined): Date | null {
  if (!value) return null

  try {
    // Handle timestamps (number or numeric string)
    if (typeof value === 'number') {
      return new Date(value)
    }

    if (typeof value === 'string') {
      // Try parsing as number first
      const asNumber = Number(value)
      if (!isNaN(asNumber)) {
        return new Date(asNumber)
      }

      // Convert SQLite format to ISO format (replace space with T)
      const isoFormat = value.replace(' ', 'T')
      const date = new Date(isoFormat)

      if (!isNaN(date.getTime())) {
        return date
      }
    }

    return null
  } catch {
    return null
  }
}

/**
 * Parse datetime and return milliseconds since epoch.
 *
 * Convenience function combining parseDateTime and getTime().
 * Returns 0 if parsing fails instead of null for numeric operations.
 *
 * @param value - Datetime string or timestamp to parse
 * @returns Milliseconds since epoch, or 0 if parsing fails
 *
 * @example
 * ```typescript
 * const ms = parseToMs('2024-01-01T12:00:00Z')
 * const elapsed = ms - 1000 // Safe: won't error if parsing failed
 * ```
 */
export function parseToMs(value: string | number | Date | null | undefined): number {
  const date = value instanceof Date ? value : parseDateTime(value)
  return date ? date.getTime() : 0
}

/**
 * Calculate elapsed time between two dates/timestamps.
 *
 * Returns the difference in milliseconds. Handles null/undefined inputs
 * gracefully by treating them as epoch (0).
 *
 * @param start - Start datetime
 * @param end - End datetime (defaults to now)
 * @returns Milliseconds elapsed, or 0 if start is invalid
 *
 * @example
 * ```typescript
 * const elapsed = elapsedMs('2024-01-01T10:00:00Z', '2024-01-01T11:30:00Z')
 * console.log(elapsed) // 5400000 (1h 30m in ms)
 *
 * const now = elapsedMs('2024-01-01T10:00:00Z') // Time since then until now
 * ```
 */
export function elapsedMs(
  start: string | number | null | undefined,
  end: string | number | Date | null | undefined = new Date()
): number {
  const startMs = parseToMs(start)
  const endMs = parseToMs(end)

  if (startMs === 0) return 0

  return Math.max(0, endMs - startMs)
}

/**
 * Check if a date is in the past.
 *
 * @param value - Date to check (defaults to now if null)
 * @returns True if date is before now
 *
 * @example
 * ```typescript
 * isPast('2024-01-01T10:00:00Z') // true (in the past)
 * isPast(Date.now() + 10000) // false (in the future)
 * ```
 */
export function isPast(value: string | number | Date | null | undefined): boolean {
  const date = value instanceof Date ? value : parseDateTime(value)
  return date ? date.getTime() < Date.now() : false
}

/**
 * Check if a date is in the future.
 *
 * @param value - Date to check
 * @returns True if date is after now
 *
 * @example
 * ```typescript
 * isFuture('2025-01-01T10:00:00Z') // true (in the future)
 * isFuture('2024-01-01T10:00:00Z') // false (in the past)
 * ```
 */
export function isFuture(value: string | number | Date | null | undefined): boolean {
  const date = value instanceof Date ? value : parseDateTime(value)
  return date ? date.getTime() > Date.now() : false
}

/**
 * Check if a date is today.
 *
 * Compares dates by calendar day (ignoring time component).
 *
 * @param value - Date to check
 * @returns True if date is today
 *
 * @example
 * ```typescript
 * isToday('2024-01-15T10:00:00Z') // true if today is 2024-01-15
 * ```
 */
export function isToday(value: string | number | Date | null | undefined): boolean {
  const date = value instanceof Date ? value : parseDateTime(value)
  if (!date) return false

  const today = new Date()
  return (
    date.getFullYear() === today.getFullYear() &&
    date.getMonth() === today.getMonth() &&
    date.getDate() === today.getDate()
  )
}

/**
 * Format date as locale-specific date string.
 *
 * Uses browser's locale for formatting. Returns empty string for null/invalid input.
 *
 * @param value - Date to format
 * @param options - Intl.DateTimeFormat options
 * @returns Formatted date string (e.g., "1/15/2024")
 *
 * @example
 * ```typescript
 * formatAsDate('2024-01-15T10:00:00Z') // "1/15/2024" (en-US)
 * formatAsDate('2024-01-15T10:00:00Z', { year: 'numeric', month: 'long', day: 'numeric' })
 * // "January 15, 2024"
 * ```
 */
export function formatAsDate(
  value: string | number | Date | null | undefined,
  options?: Intl.DateTimeFormatOptions
): string {
  const date = value instanceof Date ? value : parseDateTime(value)
  if (!date) return ''

  return date.toLocaleDateString(undefined, options)
}

/**
 * Format datetime as locale-specific string with time.
 *
 * Uses browser's locale for formatting. Includes both date and time.
 * Returns empty string for null/invalid input.
 *
 * @param value - Date to format
 * @param options - Intl.DateTimeFormat options
 * @returns Formatted datetime string (e.g., "1/15/2024, 10:30:00 AM")
 *
 * @example
 * ```typescript
 * formatAsDateTime('2024-01-15T10:30:00Z') // "1/15/2024, 10:30:00 AM" (en-US)
 * ```
 */
export function formatAsDateTime(
  value: string | number | Date | null | undefined,
  options?: Intl.DateTimeFormatOptions
): string {
  const date = value instanceof Date ? value : parseDateTime(value)
  if (!date) return ''

  return date.toLocaleString(undefined, options)
}

/**
 * Format datetime as locale-specific time string.
 *
 * Returns time portion only. Returns empty string for null/invalid input.
 *
 * @param value - Date to format
 * @param options - Intl.DateTimeFormat options
 * @returns Formatted time string (e.g., "10:30:00 AM")
 *
 * @example
 * ```typescript
 * formatAsTime('2024-01-15T10:30:00Z') // "10:30:00 AM" (en-US)
 * ```
 */
export function formatAsTime(
  value: string | number | Date | null | undefined,
  options?: Intl.DateTimeFormatOptions
): string {
  const date = value instanceof Date ? value : parseDateTime(value)
  if (!date) return ''

  return date.toLocaleTimeString(undefined, options)
}

/**
 * Format date relative to now (e.g., "2 hours ago").
 *
 * Uses `Intl.RelativeTimeFormat` for locale-aware relative formatting.
 * Returns empty string for null/invalid input.
 *
 * @param value - Date to format
 * @returns Relative time string (e.g., "2 hours ago", "in 3 days")
 *
 * @example
 * ```typescript
 * const past = new Date(Date.now() - 2 * 60 * 60 * 1000) // 2 hours ago
 * formatAsRelative(past) // "2 hours ago"
 *
 * const future = new Date(Date.now() + 3 * 24 * 60 * 60 * 1000) // 3 days from now
 * formatAsRelative(future) // "in 3 days"
 * ```
 */
export function formatAsRelative(value: string | number | Date | null | undefined): string {
  const date = value instanceof Date ? value : parseDateTime(value)
  if (!date) return ''

  const now = Date.now()
  const diff = date.getTime() - now

  if (diff === 0) return 'now'

  const absMs = Math.abs(diff)
  const sign = diff < 0 ? -1 : 1

  // Determine unit and value
  const seconds = Math.floor(absMs / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  const weeks = Math.floor(days / 7)
  const months = Math.floor(days / 30)
  const years = Math.floor(days / 365)

  try {
    const rtf = new Intl.RelativeTimeFormat(undefined, { numeric: 'auto' })

    if (years !== 0) return rtf.format(sign * years, 'year')
    if (months !== 0) return rtf.format(sign * months, 'month')
    if (weeks !== 0) return rtf.format(sign * weeks, 'week')
    if (days !== 0) return rtf.format(sign * days, 'day')
    if (hours !== 0) return rtf.format(sign * hours, 'hour')
    if (minutes !== 0) return rtf.format(sign * minutes, 'minute')

    return rtf.format(sign * seconds, 'second')
  } catch {
    // Fallback if Intl.RelativeTimeFormat not supported
    return ''
  }
}

/**
 * Get start of day for a date.
 *
 * Returns new Date with time set to 00:00:00.
 *
 * @param value - Date to process (defaults to today)
 * @returns Date at start of day
 *
 * @example
 * ```typescript
 * const start = getStartOfDay('2024-01-15T10:30:00Z')
 * console.log(start) // 2024-01-15T00:00:00.000Z
 * ```
 */
export function getStartOfDay(value?: string | number | Date): Date {
  const date = value ? (value instanceof Date ? value : parseDateTime(value)) : new Date()
  if (!date) return new Date()

  const start = new Date(date)
  start.setHours(0, 0, 0, 0)
  return start
}

/**
 * Get end of day for a date.
 *
 * Returns new Date with time set to 23:59:59.999.
 *
 * @param value - Date to process (defaults to today)
 * @returns Date at end of day
 *
 * @example
 * ```typescript
 * const end = getEndOfDay('2024-01-15T10:30:00Z')
 * console.log(end) // 2024-01-15T23:59:59.999Z
 * ```
 */
export function getEndOfDay(value?: string | number | Date): Date {
  const date = value ? (value instanceof Date ? value : parseDateTime(value)) : new Date()
  if (!date) return new Date()

  const end = new Date(date)
  end.setHours(23, 59, 59, 999)
  return end
}

/**
 * Check if two dates are on the same day.
 *
 * Compares calendar days, ignoring time component.
 *
 * @param date1 - First date
 * @param date2 - Second date
 * @returns True if dates are on the same day
 *
 * @example
 * ```typescript
 * isSameDay('2024-01-15T10:00:00Z', '2024-01-15T20:00:00Z') // true
 * isSameDay('2024-01-15T10:00:00Z', '2024-01-16T10:00:00Z') // false
 * ```
 */
export function isSameDay(
  date1: string | number | Date | null | undefined,
  date2: string | number | Date | null | undefined
): boolean {
  const d1 = date1 instanceof Date ? date1 : parseDateTime(date1)
  const d2 = date2 instanceof Date ? date2 : parseDateTime(date2)

  if (!d1 || !d2) return false

  return (
    d1.getFullYear() === d2.getFullYear() &&
    d1.getMonth() === d2.getMonth() &&
    d1.getDate() === d2.getDate()
  )
}
