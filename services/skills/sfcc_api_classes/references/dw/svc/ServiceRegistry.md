# dw.svc.ServiceRegistry

## Overview
Manages service definitions and instances. Deprecated as of version 19.10; use LocalServiceRegistry instead.

## Description
Responsible for managing Service definitions and their instances. Services are defined in Business Manager, configured with callbacks during cartridge initialization, then instantiated and called to perform operations. Replaced by LocalServiceRegistry for improved configuration flexibility.

```ts
declare class ServiceRegistry  {
	/**
	 * Configure the given serviceId with a callback.
	 * @param serviceID - Unique Service ID
	 * @param configObj - Configuration callback with methods: initServiceClient, createRequest, execute, executeOverride, parseResponse, mockCall, mockFull
	 * @returns Associated ServiceDefinition for further protocol-specific configuration
	 */
	static configure(serviceID: string, configObj: Object): ServiceDefinition
	
	/**
	 * Constructs a new instance of the given service.
	 * @param serviceID - Unique Service ID
	 * @returns Service instance
	 */
	static get(serviceID: string): Service
	
	/**
	 * Gets a Service Definition shared across all Service instances.
	 * @param serviceID - Unique Service ID
	 * @returns ServiceDefinition
	 */
	static getDefinition(serviceID: string): ServiceDefinition
	
	/**
	 * Returns the status of whether the given service has been configured with a callback.
	 * @param serviceID - Unique Service ID
	 * @returns true if configure has already been called, false otherwise
	 */
	static isConfigured(serviceID: string): boolean
}
```
