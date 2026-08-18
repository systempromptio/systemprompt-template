# dw.rpc.WebReference

## Overview
Represents a web service defined in a WSDL file, providing access to service stubs for SOAP operations via JAX-RPC framework.

## Description
Backed by JAX-RPC framework for WSDL-based web services. WSDL files placed in webreferences directory are auto-generated into script classes in the WebReferences package. Use Business Manager Services module for timeout configuration. Deprecated; use webreferences2 and WebReference2 (JAX-WS based) instead. Migration tip: use collectionType=indexed property for closer API compatibility.

```ts
declare class WebReference  {
	/**
	 * The default service of the WebReference object (Read Only)
	 */
	readonly defaultService: Stub
	
	/**
	 * Returns the default service of the WebReference object
	 * @returns the default service
	 */
	getDefaultService(): Stub
	
	/**
	 * Returns a specific service from this WebReference
	 * @param service - the service to locate
	 * @param port - the port name to use
	 * @returns a specific service from this WebReference
	 */
	getService(service: string, port: string): Stub
}
```
