# dw.system.RESTResponseMgr

## Overview
Helper methods for creating REST error and success responses for Custom REST APIs and REST-like controller implementations.

## Description
Provides helper methods for creating REST error and success responses compliant with RFC standards. Primarily intended for Custom REST APIs but can be used in any controller implementation. Note that defaults like URL prefix for `type` in `createError` methods correspond to Custom REST APIs.

```ts
declare class RESTResponseMgr  {
	/**
	 * Constructs a new RESTResponseMgr instance.
	 */
	constructor()

	/**
	 * Constructs a new RESTSuccessResponse for scenarios where response body is not expected (e.g., statusCode 204).
	 * @param statusCode - The HTTP status code of the response. Must be in range (100..299).
	 * @throws {IllegalArgumentException} If the statusCode is not in the (100..299) range.
	 */
	static createEmptySuccess(statusCode: Number): RESTSuccessResponse

	/**
	 * Constructs a new RESTErrorResponse when only statusCode is available. Type is inferred from statusCode: 400='bad-request', 401='unauthorized', 403='forbidden', 404='resource-not-found', 409='conflict', 412='precondition-failed', 429='too-many-requests', 500='internal-server-error', default='about:blank'.
	 * @param statusCode - The error code of the response. Must be in range (400..599).
	 * @throws {IllegalArgumentException} If the statusCode is not in the (400..599) range.
	 */
	static createError(statusCode: Number): RESTErrorResponse

	/**
	 * Constructs a new RESTErrorResponse with custom error type. If type is not absolute URL, it's prepended with 'https://api.commercecloud.salesforce.com/documentation/error/v1/custom-errors/'. Custom types cannot use SYSTEM error prefix.
	 * @param statusCode - The error code of the response. Must be in range (400..599).
	 * @param type - Type of the error according to RFC 9457.
	 * @throws {IllegalArgumentException} If statusCode is not in (400..599) range or if type is not valid URI or conflicts with SYSTEM error type namespace.
	 */
	static createError(statusCode: Number, type: String): RESTErrorResponse

	/**
	 * Constructs a new RESTErrorResponse with statusCode, type, and title. Omits detail.
	 * @param statusCode - The error code of the response. Must be in range (400..599).
	 * @param type - Type of the error according to RFC 9457.
	 * @param title - Human-readable summary of the error type.
	 * @throws {IllegalArgumentException} If statusCode is not in (400..599) range or if type is not valid URI or conflicts with SYSTEM error type namespace.
	 */
	static createError(statusCode: Number, type: String, title: String): RESTErrorResponse

	/**
	 * Constructs a new RESTErrorResponse with statusCode, type, title, and detail. Pass null to omit title or detail.
	 * @param statusCode - The error code of the response. Must be in range (400..599).
	 * @param type - Type of the error according to RFC 9457.
	 * @param title - Human-readable summary of the error type.
	 * @param detail - Human-readable explanation of the specific occurrence of the error.
	 * @throws {IllegalArgumentException} If statusCode is not in (400..599) range or if type is not valid URI or conflicts with SYSTEM error type namespace.
	 */
	static createError(statusCode: Number, type: String, title: String, detail: String): RESTErrorResponse

	/**
	 * Constructs a new RemoteInclude object specific for SCAPI include path. BASE_PATH and ORG_ID are automatically resolved.
	 * @param apiFamily - API Family name (e.g., 'product').
	 * @param apiName - API Name (e.g., 'shopper-products').
	 * @param apiVersion - API Version (e.g., 'v1').
	 * @param resourcePath - Resource path (e.g., 'categories/root').
	 * @param params - Query parameters (optional).
	 */
	static createScapiRemoteInclude(apiFamily: String, apiName: String, apiVersion: String, resourcePath: String, ...params: URLParameter): RemoteInclude

	/**
	 * Constructs a new RemoteInclude object specific for Storefront Controller include path.
	 * @param action - Target controller container. Hostnames in URL actions are ignored.
	 * @param params - Query parameters (optional).
	 */
	static createStorefrontControllerRemoteInclude(action: URLAction, ...params: URLParameter): RemoteInclude

	/**
	 * Constructs a new RESTSuccessResponse with body and custom status code.
	 * @param body - The body of the successful response. Must be a valid JavaScript JSON object.
	 * @param statusCode - The HTTP status code of the response. Must be in range (100..299).
	 * @throws {IllegalArgumentException} If the statusCode is not in the (100..299) range.
	 */
	static createSuccess(body: Object, statusCode: Number): RESTSuccessResponse

	/**
	 * Constructs a new RESTSuccessResponse with body. HTTP status code defaults to 200.
	 * @param body - The body of the successful response. Must be a valid JavaScript JSON object.
	 */
	static createSuccess(body: Object): RESTSuccessResponse
}
```
