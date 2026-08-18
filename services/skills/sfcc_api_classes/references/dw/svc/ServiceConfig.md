# dw.svc.ServiceConfig

## Overview
Configuration object for Services.

## Description
Provides access to service configuration including credentials, profile, and service type. Extends ExtensibleObject for custom attributes.

```ts
declare class ServiceConfig extends ExtensibleObject {
	/**
	 * The related service credentials.
	 * @readonly
	 */
	credential: ServiceCredential

	/**
	 * The unique Service ID.
	 * @readonly
	 */
	ID: string

	/**
	 * The related service profile.
	 * @readonly
	 */
	profile: ServiceProfile

	/**
	 * The type of the service, such as HTTP or SOAP.
	 * @readonly
	 */
	serviceType: string

	/**
	 * Returns the related service credentials.
	 */
	getCredential(): ServiceCredential

	/**
	 * Returns the unique Service ID.
	 */
	getID(): string

	/**
	 * Returns the related service profile.
	 */
	getProfile(): ServiceProfile

	/**
	 * Returns the type of the service, such as HTTP or SOAP.
	 */
	getServiceType(): string
}
```
