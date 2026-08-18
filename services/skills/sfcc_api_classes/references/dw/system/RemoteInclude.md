# dw.system.RemoteInclude

## Overview
Represents a remote include value assignable to JSON Object properties. Used with `RESTResponseMgr` to embed remote resource content in REST responses.

## Description
Authentication/authorization checks occur only for top-level requests, not remote includes. Create instances via `RESTResponseMgr.createScapiRemoteInclude()` (SCAPI URLs only) or `RESTResponseMgr.createStorefrontControllerRemoteInclude()` (Controller URLs only). Correct rendering requires processing by `RESTSuccessResponse.render()`.

Error handling: SCAPI returns empty JSON `{}` on 404, 500 Internal Server Error for other non-2xx responses. Controllers return empty string on any non-200 response (may produce invalid JSON).

```ts
declare class RemoteInclude  {
	/**
	 * The URL string value specified for the current instance.
	 */
	readonly url: string

	readonly value: string

	/**
	 * Returns the URL string value specified for the current instance.
	 */
	getUrl(): string

	/**
	 * Returns the URL string value specified for the current instance, same as getUrl().
	 */
	toString(): string

	/**
	 * Returns the URL string value specified for the current instance, same as getUrl().
	 */
	valueOf(): Object
}
```
