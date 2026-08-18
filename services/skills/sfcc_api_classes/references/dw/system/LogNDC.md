# dw.system.LogNDC

## Overview
Nested Diagnostic Context for distinguishing interleaved log output from concurrent script executions.

## Description
Stack-based context tracking for log messages. Helps identify source of log entries when server processes multiple script calls simultaneously. NDC is automatically cleared after each script execution.

```ts
declare class LogNDC  {
	/**
	 * Views last diagnostic context without removing it; returns empty string if none exists
	 */
	peek(): string

	/**
	 * Removes and returns last diagnostic context; returns empty string if none exists
	 */
	pop(): string

	/**
	 * Pushes new diagnostic context for current script execution
	 * @param message - New diagnostic context information
	 */
	push(message: string): void

	/**
	 * Clears all diagnostic context for this script call
	 */
	remove(): void
}
```
