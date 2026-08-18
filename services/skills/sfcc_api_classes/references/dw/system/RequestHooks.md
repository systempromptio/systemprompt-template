# dw.system.RequestHooks

## Overview
Script hooks for receiving notifications about storefront requests. Hook functions must be exported from scripts in site cartridges and registered via `hooks.json`.

## Description
Register hooks in `package.json` with `"hooks": "./hooks.json"` entry. The hooks.json file lists all registered hooks with `name` (extension point) and `script` (path to exported hook function) properties.

```ts
declare class RequestHooks {
	/**
	 * Extension point name dw.system.request.onRequest.
	 */
	static extensionPointOnRequest: 'dw.system.request.onRequest'

	/**
	 * Extension point name dw.system.request.onSession.
	 */
	static extensionPointOnSession: 'dw.system.request.onSession'

	/**
	 * Called when a storefront request was received from the client.
	 * @returns Status.OK for success, Status.ERROR for error
	 */
	onRequest(): Status

	/**
	 * Called when a new storefront session was started.
	 * @returns Status.OK for success, Status.ERROR for error
	 */
	onSession(): Status
}
```
