# dw.svc.LocalServiceRegistry

## Overview
Manages service instances configured in Business Manager.

## Description
The LocalServiceRegistry is responsible for managing Service instances. Typical usage involves several steps: 1) The service is defined in the Business Manager and configured with necessary credentials. 2) An instance of the service is created and configured in a script. 3) The service is called in order to perform the operation. Unlike ServiceRegistry, the configured service is local to the current script call, so this deals directly with Service instances rather than the intermediate ServiceDefinition. This means that a cartridge-level initialization script (and the package.json) is no longer needed. See ServiceCallback for all the callback options, and individual Service classes for customization specific to a service type.

```
Object
  dw.svc.LocalServiceRegistry
```

```ts
declare class LocalServiceRegistry  {
	/**
	 * Constructs and configures a service with a callback.
	 * @param serviceID - Unique Service ID.
	 * @param configObj - Configuration callback. See ServiceCallback for a description of available callback methods.
	 * @returns Associated Service, which can be used for further protocol-specific configuration.
	 */
	static createService(serviceID: string, configObj: Object): Service
}
```
