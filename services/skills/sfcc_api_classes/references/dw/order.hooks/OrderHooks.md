# dw.order.hooks.OrderHooks

## Overview
Interface for script hooks to customize order logic.

## Description
Represents all script hooks that can be registered to customize the order logic. Contains extension points (hook names) and functions called by each extension point. Hook functions must be defined inside a JavaScript source, exported, and registered via `hooks.json` in a site cartridge's `package.json`.

```ts
declare class OrderHooks  {
	/**
	 * Extension point name for creating order numbers.
	 */
	static extensionPointCreateOrderNo: 'dw.order.createOrderNo'

	/**
	 * Creates a new order number.
	 */
	createOrderNo(): string
}
```
