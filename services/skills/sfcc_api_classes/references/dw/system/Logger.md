# dw.system.Logger

## Overview
Static utility class for obtaining Log instances and performing direct logging operations.

## Description
Factory for Log objects. Use getRootLogger() for root logger, getLogger(String) for category-specific loggers, or getLogger(String, String) for custom file loggers. Also provides static convenience methods for direct logging.

```ts
declare class Logger  {
	/**
	 * True if debug logging is enabled
	 * @readonly
	 */
	readonly debugEnabled: boolean

	/**
	 * True if error logging is enabled
	 * @readonly
	 */
	readonly errorEnabled: boolean

	/**
	 * True if info logging is enabled
	 * @readonly
	 */
	readonly infoEnabled: boolean

	/**
	 * Root logger instance
	 * @readonly
	 */
	readonly rootLogger: Log

	/**
	 * True if warning logging is enabled
	 * @readonly
	 */
	readonly warnEnabled: boolean

	/**
	 * Reports debug-level message with MessageFormat-style argument substitution
	 * @param msg - Message to log
	 * @param args - Arguments to embed in message
	 */
	static debug(msg: string, ...args: Object[]): void

	/**
	 * Reports error-level message with MessageFormat-style argument substitution
	 * @param msg - Message to log
	 * @param args - Arguments to embed in message
	 */
	static error(msg: string, ...args: Object[]): void

	/**
	 * Returns logger for specified category
	 * @param category - Logger category
	 */
	static getLogger(category: string): Log

	/**
	 * Returns logger for custom file with specified prefix and category; throws if daily custom log file limit exceeded
	 * @param fileNamePrefix - File name prefix (3-25 chars, alphanumeric/dash/underscore only, must start/end with alphanumeric)
	 * @param category - Logger category
	 */
	static getLogger(fileNamePrefix: string, category: string): Log

	/**
	 * Returns the root logger
	 */
	static getRootLogger(): Log

	/**
	 * Reports info-level message with MessageFormat-style argument substitution
	 * @param msg - Message to log
	 * @param args - Arguments to embed in message
	 */
	static info(msg: string, ...args: Object[]): void

	/**
	 * Returns true if debug logging is enabled
	 */
	static isDebugEnabled(): boolean

	/**
	 * Returns true if error logging is enabled
	 */
	static isErrorEnabled(): boolean

	/**
	 * Returns true if info logging is enabled
	 */
	static isInfoEnabled(): boolean

	/**
	 * Returns true if warn logging is enabled
	 */
	static isWarnEnabled(): boolean

	/**
	 * Reports warn-level message with MessageFormat-style argument substitution
	 * @param msg - Message to log
	 * @param args - Arguments to embed in message
	 */
	static warn(msg: string, ...args: Object[]): void
}
```
