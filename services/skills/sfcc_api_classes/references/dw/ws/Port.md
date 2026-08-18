# dw.ws.Port

## Overview
Represents a port to a Service Endpoint Interface.

## Description
Represents a port to a Service Endpoint Interface. Provides access to operations the service provides. Use WSUtil class to perform operations on the port such as setting timeout values and configuring security.

Developers should set a low timeout to ensure responsiveness and avoid thread exhaustion. Default request timeout is 15 minutes for jobs, 2 minutes otherwise. If calling script timeout is lower, script timeout is used.

```ts
declare class Port  {
	/**
	 * Property constant for controlling the content type encoding of an outgoing message.
	 */
	static ENCODING: string

	/**
	 * The target service endpoint address. URI scheme must correspond to the protocol/transport binding.
	 */
	static ENDPOINT_ADDRESS_PROPERTY: string

	/**
	 * Password for authentication. Used with USERNAME_PROPERTY.
	 */
	static PASSWORD_PROPERTY: string

	/**
	 * Boolean property indicating whether client wants to participate in a session with service endpoint. Default is false.
	 */
	static SESSION_MAINTAIN_PROPERTY: string

	/**
	 * User name for authentication. Used with PASSWORD_PROPERTY.
	 */
	static USERNAME_PROPERTY: string

	/**
	 * Port constructor.
	 */
	constructor()
}
```
