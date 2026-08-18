# dw.svc.ServiceCallback

## Overview
Defines callbacks for use with the LocalServiceRegistry.

## Description
Provides callback methods invoked during service execution. Not used directly; exists for documentation of available methods. These methods are called in sequence: initServiceClient, createRequest, execute, parseResponse. For mocked services, mockFull or mockCall replace this sequence. Implementations control request formation, execution, and response parsing for different service types (HTTP, SOAP, FTP, GENERIC).

```ts
declare class ServiceCallback  {
	/**
	 * Overrides the URL provided by the service configuration. Usually better to call Service.setURL() within createRequest() for dynamic modification.
	 * @readonly
	 */
	URL: string

	/**
	 * Creates a request object to be used when calling the service. Required unless execute method is implemented. The type of object expected is dependent on the service.
	 * @param service - Service being executed.
	 * @param params - Parameters given to the call method.
	 */
	createRequest(service: Service, ...params: Object[]): Object

	/**
	 * Provides service-specific execution logic. Can be overridden to execute FTP commands chain or perform actual remote call on a webservice stub.
	 * @param service - Service being executed.
	 * @param request - Request object returned by createRequest.
	 * @throws Exception
	 */
	execute(service: Service, request: Object): Object

	/**
	 * Allows filtering communication URL, request, and response log messages. If not implemented, no filtering is performed and message is logged as-is.
	 * @param msg - Original log message.
	 */
	filterLogMessage(msg: string): string

	/**
	 * Creates a communication log message for the given request. If not implemented, default logic converts request into a log message.
	 * @param request - Request object.
	 */
	getRequestLogMessage(request: Object): string

	/**
	 * Creates a response log message for the given request. If not implemented, default logic converts response into a log message.
	 * @param response - Response object.
	 */
	getResponseLogMessage(response: Object): string

	/**
	 * Returns URL override. Default behavior uses URL from service configuration. Usually better to call Service.setURL() within createRequest().
	 */
	getURL(): string

	/**
	 * Creates a protocol-specific client object. Does not normally need to be implemented except for SOAP services or to override HTTP service default configuration.
	 * @param service - Service object.
	 * @throws Exception
	 */
	initServiceClient(service: Service): Object

	/**
	 * Override to mock the remote portion of the service call. Replaces only the execute method when service is in mock mode.
	 * @param service - Service being executed.
	 * @param requestObj - Request object returned by createRequest.
	 */
	mockCall(service: Service, requestObj: Object): Object

	/**
	 * Override to mock the entire service call, including createRequest, execute, and parseResponse phases. Takes precedence over mockCall when service is in mock mode.
	 * @param service - Service being executed.
	 * @param args - Arguments given to the call method.
	 */
	mockFull(service: Service, ...args: Object[]): Object

	/**
	 * Creates a response object from a successful service call. Required unless execute method is implemented.
	 * @param service - Service being executed.
	 * @param response - Response object from execute method or underlying service call.
	 */
	parseResponse(service: Service, response: Object): Object
}
```
