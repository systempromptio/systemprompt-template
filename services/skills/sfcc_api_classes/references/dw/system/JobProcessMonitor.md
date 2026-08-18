# dw.system.JobProcessMonitor

## Overview
**Reserved for future use** - Provides job process monitoring capabilities.

## Description
Reserved for future use.

```
Object
  dw.system.JobProcessMonitor
```

```ts
declare class JobProcessMonitor {
	/**
	 * Reserved for future use. Gets the total work count.
	 * @readonly
	 */
	readonly totalWork: number

	/**
	 * Reserved for future use. Gets the work message.
	 * @readonly
	 */
	readonly workMessage: string

	/**
	 * Reserved for future use. Gets the total work count.
	 * @returns The total work count
	 */
	getTotalWork(): number

	/**
	 * Reserved for future use. Gets the work message.
	 * @returns The work message
	 */
	getWorkMessage(): string

	/**
	 * Reserved for future use. Sets the total work count.
	 * @param totalWork - The total work count
	 */
	setTotalWork(totalWork: number): void

	/**
	 * Reserved for future use. Sets the work message.
	 * @param msg - The message to use. If msg is null, then an empty string will be used
	 */
	setWorkMessage(msg: string): void

	/**
	 * Reserved for future use. Increments the count of work items by the value of the specified parameter.
	 * @param worked - The number of items worked
	 */
	worked(worked: number): void
}
```
