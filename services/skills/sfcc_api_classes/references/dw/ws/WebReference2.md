# dw.ws.WebReference2

## Overview
Represents a web service defined in a WSDL file, backed by JAX-WS framework. Supports document/literal encoding for accessing SOAP web services.

## Description
Represents a web service defined in a WSDL file. The implementation is backed by a JAX-WS framework.

This implementation does not support RPC/encoded WSDLs. Such a WSDL must be migrated to a supported encoding such as Document/literal to work with this API.

To create an instance of a WebReference2, you put a web service WSDL file in the webreferences2 directory and reference the WSDL file in a B2C Commerce Script. You then request the service Port using one of the get service methods. For example, if your WSDL file is MyWSDL.wsdl, here is how you create an instance of WebReference2 and access the Port:

```javascript
var webref : WebReference2 = webreferences2.MyWSDL;
var port : Port = webref.getDefaultService();
```

Note that all script classes representing your WSDL file are placed in the webreferences2 package. To use classes in the webreferences2 package, you do not need to use the importPackage statement in your B2C Commerce Script file.

The generated API may be customized via a property file named `<WSDLFile>.properties`. Supported properties include namespace, underscoreBinding, collectionType, enableWrapperStyle, and various logging configuration options.

The messages sent to and from the remote server are logged at DEBUG level on sandboxes, and not logged at all on production. The custom log category used is derived from the WSDL name and message type (e.g., webreferences2.MyWSDL.request and webreferences2.MyWSDL.response).

```
Object
  dw.ws.WebReference2
```

```ts
declare class WebReference2 {
	/**
	 * The default service endpoint interface port of the web reference. The default service is determined as the first service based on the alphabetic order of the service name, and within the service the first SOAP port based on the alphabetic order of the port name.
	 */
	readonly defaultService: Port

	constructor()

	/**
	 * Returns the default service endpoint interface port of the web reference. The default service is determined as the first service based on the alphabetic order of the service name, and within the service the first SOAP port based on the alphabetic order of the port name.
	 * @returns the default service of the web reference
	 */
	getDefaultService(): Port

	/**
	 * Returns a specific service from this web reference.
	 * @param service - the service to locate
	 * @param portName - the name of the port to use
	 * @returns a specific service from this web reference
	 */
	getService(service: string, portName: string): Port
}
```
