# dw.order.hooks.ReturnHooks

## Overview
Interface for script hooks to customize order center return resource.

## Description
Represents all script hooks that can be registered to customize the order center return resource. Contains extension points (hook names) and functions called by each extension point. Hook functions must be defined inside a JavaScript source, exported, and registered via `hooks.json` in a site cartridge's `package.json`.

```ts
declare class ReturnHooks  {
	/**
	 * Extension point name for adding return item.
	 */
	static extensionPointAddReturnItem: 'dw.order.return.addReturnItem'

	/**
	 * Extension point name for after status change.
	 */
	static extensionPointAfterStatusChange: 'dw.order.return.afterStatusChange'

	/**
	 * Extension point name for changing status.
	 */
	static extensionPointChangeStatus: 'dw.order.return.changeStatus'

	/**
	 * Extension point name for creating return.
	 */
	static extensionPointCreateReturn: 'dw.order.return.createReturn'

	/**
	 * Extension point name for notifying status change.
	 */
	static extensionPointNotifyStatusChange: 'dw.order.return.notifyStatusChange'

	/**
	 * Adds a return item to a return.
	 * @param retrn - the return for which a return item should be created
	 * @param inputData - the return item data
	 */
	addReturnItem(retrn: Return, inputData: unknown): Status

	/**
	 * Called after changeStatus returns Status.OK, runs in separate transaction.
	 * @param retrn - the return
	 */
	afterStatusChange(retrn: Return): Status

	/**
	 * Changes the status of a return.
	 * @param retrn - the return which status should change
	 * @param inputData - the data in which the new status is included
	 */
	changeStatus(retrn: Return, inputData: unknown): Status

	/**
	 * Creates a new return based on a return case.
	 * @param inputData - the return data
	 */
	createReturn(inputData: unknown): Return

	/**
	 * Notifies of successful status change, called outside any transaction.
	 * @param retrn - the return
	 */
	notifyStatusChange(retrn: Return): Status
}
```
