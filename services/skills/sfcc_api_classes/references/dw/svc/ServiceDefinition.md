# dw.svc.ServiceDefinition

## Overview
Base class for service definitions representing shared configuration across Service instances. Deprecated as of version 19.10.

## Description
Represents configuration shared across all Service instances. A service definition stores service-wide settings including callbacks for custom logic. Use LocalServiceRegistry instead which allows direct Service configuration.

## All Known Subclasses
FTPServiceDefinition, HTTPFormServiceDefinition, HTTPServiceDefinition, SOAPServiceDefinition

```ts
declare class ServiceDefinition  {
	/**
	 * Service configuration stored in the database.
	 * @readonly
	 */
	readonly configuration: ServiceConfig
	
	/**
	 * Mock mode status for all instances of this definition.
	 */
	mock: boolean
	
	/**
	 * Name of this service.
	 * @readonly
	 */
	readonly serviceName: string
	
	/**
	 * Status of whether the shared throwOnError flag is set.
	 */
	throwOnError: boolean
	
	/**
	 * Register a callback to handle custom portions of the service.
	 * @param config - Callback object with methods: initServiceClient, createRequest, execute, executeOverride, parseResponse, mockCall, mockFull
	 * @returns this
	 */
	configure(config: Object): ServiceDefinition
	
	/**
	 * Returns the Service Configuration stored in the database.
	 * @returns Service Configuration
	 */
	getConfiguration(): ServiceConfig
	
	/**
	 * Returns the name of this service.
	 * @returns Service name
	 */
	getServiceName(): string
	
	/**
	 * Returns the status of whether mock mode is enabled for all instances of this definition.
	 * @returns true for mock mode, false otherwise
	 */
	isMock(): boolean
	
	/**
	 * Returns the status of whether the shared throwOnError flag is set.
	 * @returns throwOnError flag
	 */
	isThrowOnError(): boolean
	
	/**
	 * Sets the mock mode for all Service instances that use this definition.
	 * @returns this Service Definition
	 */
	setMock(): ServiceDefinition
	
	/**
	 * Sets the throwOnError flag to true for all Service instances that use this definition.
	 * @returns this Service Definition
	 */
	setThrowOnError(): ServiceDefinition
}
```
