# dw.system.Log

## Overview
Log4j-style logger for script-level logging across debug, info, warn, error, and fatal severity levels.

## Description
Obtain logger instances via Logger.getRootLogger(), Logger.getLogger(String), or Logger.getLogger(String, String). Supports message formatting with embedded arguments using Java MessageFormat syntax (e.g., "Failure {0} in {1}").

```ts
declare class Log  {
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
	 * Nested Diagnostic Context for this script call
	 * @readonly
	 */
	readonly NDC: LogNDC

	/**
	 * True if warn logging is enabled
	 * @readonly
	 */
	readonly warnEnabled: boolean

	/**
	 * Reports debug-level message with MessageFormat-style argument substitution
	 * @param msg - Message to log
	 * @param args - Arguments to embed in message
	 */
	debug(msg: string, ...args: Object[]): void

	/**
	 * Reports error-level message with MessageFormat-style argument substitution
	 * @param msg - Message to log
	 * @param args - Arguments to embed in message
	 */
	error(msg: string, ...args: Object[]): void

	/**
	 * Reports fatal-level message (always enabled, optionally sent via email)
	 * @param msg - Message to log
	 * @param args - Arguments to embed in message
	 */
	fatal(msg: string, ...args: Object[]): void

	/**
	 * Returns the Nested Diagnostic Context for this script call
	 */
	static getNDC(): LogNDC

	/**
	 * Reports info-level message with MessageFormat-style argument substitution
	 * @param msg - Message to log
	 * @param args - Arguments to embed in message
	 */
	info(msg: string, ...args: Object[]): void

	/**
	 * Returns true if debug logging is enabled
	 */
	isDebugEnabled(): boolean

	/**
	 * Returns true if error logging is enabled
	 */
	isErrorEnabled(): boolean

	/**
	 * Returns true if info logging is enabled
	 */
	isInfoEnabled(): boolean

	/**
	 * Returns true if warn logging is enabled
	 */
	isWarnEnabled(): boolean

	/**
	 * Reports warn-level message with MessageFormat-style argument substitution
	 * @param msg - Message to log
	 * @param args - Arguments to embed in message
	 */
	warn(msg: string, ...args: Object[]): void
}
```
