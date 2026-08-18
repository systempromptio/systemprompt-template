# dw.svc.SOAPService

## Overview
Represents a SOAP WebService for making SOAP-based service calls.

## Description
Provides SOAP WebService functionality with authentication and service client configuration. Supports BASIC and NONE authentication types, with BASIC as default.

```ts
declare class SOAPService extends Service {
	/**
	 * Authentication type for the SOAP service.
	 */
	authentication: string
	
	/**
	 * serviceClient object for SOAP operations.
	 */
	serviceClient: Object
	
	/**
	 * Returns the authentication type.
	 * @returns Authentication type
	 */
	getAuthentication(): string
	
	/**
	 * Returns the serviceClient object.
	 * @returns serviceClient object
	 */
	getServiceClient(): Object
	
	/**
	 * Sets the type of authentication. Valid values: "BASIC" and "NONE". Default is BASIC.
	 * @param authentication - Type of authentication
	 * @returns this SOAP WebService
	 */
	setAuthentication(authentication: string): SOAPService
	
	/**
	 * Sets the serviceClient object. Must be set in the prepareCall method prior to execute being called.
	 * @param o - serviceClient object
	 * @returns this SOAP WebService
	 */
	setServiceClient(o: Object): SOAPService
}
```
