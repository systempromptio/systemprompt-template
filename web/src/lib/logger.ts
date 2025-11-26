/**
 * Centralized Logging System
 *
 * Environment-aware, structured logging with log levels and module filtering.
 * Production safe with automatic output suppression for non-critical levels.
 *
 * Log Levels:
 * - DEBUG (0): Detailed debugging information (development only)
 * - INFO (1): Informational messages (development)
 * - WARN (2): Recoverable issues (all environments)
 * - ERROR (3): Critical failures (all environments)
 * - NONE (4): Disable all logging
 *
 * Configuration via environment:
 * - VITE_LOG_LEVEL: Set log level threshold (debug|info|warn|error|none)
 * - VITE_LOG_MODULES: Comma-separated module names to filter (e.g., "schema/resolver,a2a/client")
 *
 * @example
 * ```ts
 * import { logger } from '@/lib/logger'
 *
 * logger.debug('User action', { userId: '123', action: 'login' }, 'auth')
 * logger.warn('Rate limit approaching', { remaining: 5 }, 'api')
 * logger.error('Database connection failed', error, 'database')
 * ```
 */

/**
 * Standard log levels ordered by severity (ascending)
 *
 * @example
 * ```ts
 * const level = LogLevel.WARN
 * if (level >= LogLevel.ERROR) handleCritical()
 * ```
 */
export const LogLevel = {
  DEBUG: 0,
  INFO: 1,
  WARN: 2,
  ERROR: 3,
  NONE: 4,
} as const

type LogLevelValue = typeof LogLevel[keyof typeof LogLevel]

interface LoggerConfig {
  level: LogLevelValue
  enabledModules?: string[]
}

/**
 * Singleton logger instance with environment-aware configuration
 *
 * Defaults:
 * - Development: INFO level
 * - Production: WARN level
 * - Respects VITE_LOG_LEVEL and VITE_LOG_MODULES overrides
 */
class Logger {
  private config: LoggerConfig

  constructor() {
    this.config = this.getConfig()
  }

  /**
   * Resolves logger configuration from environment variables
   *
   * Applies precedence:
   * 1. VITE_LOG_LEVEL environment variable (if set)
   * 2. Development defaults (INFO)
   * 3. Production defaults (WARN)
   *
   * @returns Resolved logger configuration
   */
  private getConfig(): LoggerConfig {
    const isDev = import.meta.env.DEV
    const logLevel = import.meta.env.VITE_LOG_LEVEL

    let level: LogLevelValue = isDev ? LogLevel.INFO : LogLevel.WARN

    if (logLevel === 'debug') level = LogLevel.DEBUG
    else if (logLevel === 'info') level = LogLevel.INFO
    else if (logLevel === 'warn') level = LogLevel.WARN
    else if (logLevel === 'error') level = LogLevel.ERROR
    else if (logLevel === 'none') level = LogLevel.NONE

    return {
      level,
      enabledModules: import.meta.env.VITE_LOG_MODULES?.split(','),
    }
  }

  /**
   * Determines if a log message should be output
   *
   * Checks both log level threshold and module filter list.
   * Returns false if:
   * - Message level is below configured threshold, OR
   * - Module filter is set and module name is not included
   *
   * @param level - Log level of the message
   * @param module - Module name for context and filtering
   * @returns True if message should be logged
   */
  private shouldLog(level: LogLevelValue, module?: string): boolean {
    if (level < this.config.level) return false
    if (this.config.enabledModules && module) {
      return this.config.enabledModules.includes(module)
    }
    return true
  }

  /**
   * Formats a log message with timestamp, level, and module context
   *
   * Format: `HH:MM:SS LEVEL [module] message`
   *
   * @param level - Log level label
   * @param module - Source module name
   * @param message - Log message text
   * @returns Formatted log message prefix
   */
  private format(level: string, module: string | undefined, message: string): string {
    const timestamp = new Date().toISOString().split('T')[1].split('.')[0]
    const moduleStr = module ? `[${module}]` : ''
    return `${timestamp} ${level} ${moduleStr} ${message}`
  }

  /**
   * Logs a debug message with optional structured data
   *
   * Only output in development or when VITE_LOG_LEVEL=debug.
   * Useful for detailed troubleshooting during development.
   *
   * @param message - Human-readable log message
   * @param data - Optional structured data to include (objects, arrays, etc.)
   * @param module - Module name for context and filtering
   *
   * @example
   * ```ts
   * logger.debug('Processing request', { requestId: '123', path: '/api/users' }, 'api')
   * ```
   */
  debug(message: string, data?: unknown, module?: string): void {
    if (!this.shouldLog(LogLevel.DEBUG, module)) return
    if (data !== undefined) {
      console.debug(this.format('DEBUG', module, message), data)
    } else {
      console.debug(this.format('DEBUG', module, message))
    }
  }

  /**
   * Logs an informational message with optional structured data
   *
   * Output in development mode or when VITE_LOG_LEVEL=info or lower.
   * Use for important state changes and system events.
   *
   * @param message - Human-readable log message
   * @param data - Optional structured data to include
   * @param module - Module name for context and filtering
   *
   * @example
   * ```ts
   * logger.info('Server started', { port: 3000, env: 'production' }, 'server')
   * ```
   */
  info(message: string, data?: unknown, module?: string): void {
    if (!this.shouldLog(LogLevel.INFO, module)) return
    if (data !== undefined) {
      console.info(this.format('INFO', module, message), data)
    } else {
      console.info(this.format('INFO', module, message))
    }
  }

  /**
   * Logs a warning about a recoverable issue with optional data
   *
   * Output in all environments unless VITE_LOG_LEVEL=error or above.
   * Use for degraded functionality, missing optional configuration, etc.
   *
   * @param message - Human-readable warning message
   * @param data - Optional structured data to include
   * @param module - Module name for context and filtering
   *
   * @example
   * ```ts
   * logger.warn('Fallback used', { feature: 'analytics', reason: 'not configured' }, 'config')
   * ```
   */
  warn(message: string, data?: unknown, module?: string): void {
    if (!this.shouldLog(LogLevel.WARN, module)) return
    if (data !== undefined) {
      console.warn(this.format('WARN', module, message), data)
    } else {
      console.warn(this.format('WARN', module, message))
    }
  }

  /**
   * Logs a critical error that requires attention
   *
   * Output in all environments unless VITE_LOG_LEVEL=none.
   * Always use for exceptions, failed operations, and recoveryneeded situations.
   *
   * @param message - Human-readable error message
   * @param error - Error object or value that caused the failure
   * @param module - Module name for context and filtering
   *
   * @example
   * ```ts
   * try {
   *   await fetchData()
   * } catch (error) {
   *   logger.error('Data fetch failed', error, 'data-loader')
   * }
   * ```
   */
  error(message: string, error?: unknown, module?: string): void {
    if (!this.shouldLog(LogLevel.ERROR, module)) return
    if (error !== undefined) {
      console.error(this.format('ERROR', module, message), error)
    } else {
      console.error(this.format('ERROR', module, message))
    }
  }

  /**
   * Changes the global log level threshold at runtime
   *
   * Useful for enabling debug logging temporarily during development
   * or production troubleshooting without reloading the application.
   *
   * @param level - New log level threshold
   *
   * @example
   * ```ts
   * if (debugMode) logger.setLevel(LogLevel.DEBUG)
   * ```
   */
  setLevel(level: LogLevelValue): void {
    this.config.level = level
  }
}

/**
 * Global logger singleton instance
 *
 * Automatically configured based on environment and build flags.
 * Available throughout the application for structured logging.
 *
 * @example
 * ```ts
 * import { logger } from '@/lib/logger'
 * logger.info('Application ready', { version: '1.0.0' }, 'app')
 * ```
 */
export const logger = new Logger()
