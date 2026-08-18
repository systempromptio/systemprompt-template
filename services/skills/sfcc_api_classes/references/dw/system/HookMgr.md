# dw.system.HookMgr

## Overview
Provides functionality to call hooks, extension points in business logic where scripts can be registered to customize functionality.

## Description
This class provides functionality to call hooks. A hook is an extension point in the business logic, where you can register scripts to customize functionality.

```
Object
  dw.system.HookMgr
```

```ts
declare class HookMgr {
	/**
	 * Calls a hook based on the specified extensionPoint and function.
	 * If a hook throws an exception, then this method will also throw an exception.
	 * If no hook and no system default implementation is provided, then this method will return undefined.
	 * @param extensionPoint - The extension point to call
	 * @param function - The script function to call
	 * @param args - The Array of function parameters
	 * @returns The object returned by the hook or undefined
	 */
	static callHook(extensionPoint: string, function: string, ...args: Object): Object

	/**
	 * Checks whether a hook is registered or a system default implementation exists for this extension point.
	 * extensionPoint refers to the same name used to register a script as implementation. With this method it's only
	 * possible to check for a whole script registered but it is not possible to check whether an individual function is implemented.
	 * @param extensionPoint - The extension point
	 * @returns true if a hook is registered or a default implementation exists, otherwise false
	 */
	static hasHook(extensionPoint: string): boolean
}
```
