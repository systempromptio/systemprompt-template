# dw.system.RESTSuccessResponse

## Overview
REST success response compliant with RFC standards, created via RESTResponseMgr.

## Description
Represents a REST success response compliant with RFC standards. Instantiated only via `createSuccess` methods in RESTResponseMgr.

```ts
declare class RESTSuccessResponse  {
	/**
	 * Sends the RESTSuccessResponse object as HTTP response to the client. Sets "Content-Type" to "application/json" and expects body to be a valid JavaScript JSON object.
	 * @throws {IllegalStateException} If the RESTSuccessResponse object is already rendered.
	 * @throws {Exception} If there is an error while serializing the body.
	 */
	render(): void
}
```
