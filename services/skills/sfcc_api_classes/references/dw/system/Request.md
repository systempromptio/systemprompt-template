# dw.system.Request

## Overview
Represents a request in Commerce Cloud Digital. Each pipeline dictionary contains a `CurrentRequest` object.

## Description
Most requests are HTTP requests. Use this object to get HTTP headers, cookies, parameters, and other request information. For job-initiated requests, HTTP-related methods return null.

```ts
declare class Request  {
	/**
	 * The client id of the current SCAPI or OCAPI request. Returns null if not SCAPI/OCAPI. For Commerce Cloud owned client ids, an alias is returned.
	 * @readonly
	 */
	readonly clientId: string

	/**
	 * All custom attributes associated with the request. Stored for request lifetime.
	 * @readonly
	 */
	readonly custom: CustomAttributes

	/**
	 * Physical location for current request based on IP address. Returns null if geolocation tracking feature is disabled.
	 */
	geolocation: Geolocation

	/**
	 * Cookies object for reading cookies sent by the client. Use Response.addHttpCookie() to add cookies to outgoing response.
	 * @readonly
	 */
	readonly httpCookies: Cookies

	/**
	 * Map containing all HTTP header values.
	 * @readonly
	 */
	readonly httpHeaders: Map

	/**
	 * Host name or null if no host name.
	 * @readonly
	 */
	readonly httpHost: string

	/**
	 * Locale from HTTP header or null if no associated locale.
	 * @readonly
	 */
	readonly httpLocale: string

	/**
	 * HTTP method name (GET, POST, PUT, etc.).
	 * @readonly
	 */
	readonly httpMethod: string

	/**
	 * Parameter map containing HTTP parameters for current request.
	 * @readonly
	 */
	readonly httpParameterMap: HttpParameterMap

	/**
	 * Map containing raw HTTP parameters. Name/value pairs where each name is String and each value is String array.
	 * @readonly
	 */
	readonly httpParameters: Map

	/**
	 * The path.
	 * @readonly
	 */
	readonly httpPath: string

	/**
	 * HTTP protocol used ("http" or "https"). Returns null if not HTTP request (e.g., job).
	 * @readonly
	 */
	readonly httpProtocol: string

	/**
	 * Query string or null if no query string.
	 * @readonly
	 */
	readonly httpQueryString: string

	/**
	 * Referer or null if no referer.
	 * @readonly
	 */
	readonly httpReferer: string

	/**
	 * Remote address or null if not found.
	 * @readonly
	 */
	readonly httpRemoteAddress: string

	/**
	 * Identifies if this is an HTTP request.
	 * @deprecated Effectively always returns true.
	 * @readonly
	 */
	readonly httpRequest: boolean

	/**
	 * Whether HTTP communication is secure (https). Returns false if not HTTP request.
	 * @readonly
	 */
	readonly httpSecure: boolean

	/**
	 * Complete URL received at server. Does not include SEO optimizations.
	 * @readonly
	 */
	readonly httpURL: URL

	/**
	 * HTTP user agent or null if no user agent.
	 * @readonly
	 */
	readonly httpUserAgent: string

	/**
	 * True if request is for remote include, false if top-level request.
	 * @readonly
	 */
	readonly includeRequest: boolean

	/**
	 * Locale of current request set by system based on URL. May differ from getHttpLocale() (user agent's preferred locale).
	 */
	locale: string

	/**
	 * OCAPI version of current request. Returns null if not OCAPI request.
	 * @readonly
	 */
	readonly ocapiVersion: string

	/**
	 * Page meta data associated with current request.
	 * @readonly
	 */
	readonly pageMetaData: PageMetaData

	/**
	 * Unique identifier of current request. Helpful for debugging to relate debug messages to particular request.
	 * @readonly
	 */
	readonly requestID: string

	/**
	 * Whether request originated in SCAPI.
	 * @readonly
	 */
	readonly SCAPI: boolean

	/**
	 * Map of SCAPI path parameters (keys: parameter names from pattern, values: parameter values from request). Returns null if not SCAPI request.
	 * @readonly
	 */
	readonly SCAPIPathParameters: Map

	/**
	 * SCAPI path pattern (/api-family/api-name/version/organizations/{organizationId}/resource/{id}). Returns null if not SCAPI request.
	 * @readonly
	 */
	readonly SCAPIPathPattern: string

	/**
	 * Session associated with this request.
	 * @readonly
	 */
	readonly session: Session

	/**
	 * Form submitted by client if request represents form submission.
	 * @readonly
	 */
	readonly triggeredForm: Form

	/**
	 * Form action triggered by client if request represents form submission.
	 * @readonly
	 */
	readonly triggeredFormAction: FormAction

	/**
	 * Adds cookie to outgoing response. Can be called multiple times. Last-set cookie with same name/domain/path wins. Set maxAge to 0 to delete cookie.
	 * @deprecated Use Response.addHttpCookie(Cookie) instead.
	 * @param cookie - a Cookie object
	 */
	addHttpCookie(cookie: Cookie): void

	/**
	 * Returns client id of current SCAPI or OCAPI request.
	 */
	getClientId(): string

	/**
	 * Returns all custom attributes associated with the request.
	 */
	getCustom(): CustomAttributes

	/**
	 * Returns physical location for current request if available.
	 */
	getGeolocation(): Geolocation

	/**
	 * Returns Cookies object for reading cookies sent by client.
	 */
	getHttpCookies(): Cookies

	/**
	 * Returns Map containing all HTTP header values.
	 */
	getHttpHeaders(): Map

	/**
	 * Returns host name or null if no host name.
	 */
	getHttpHost(): string

	/**
	 * Returns locale or null if no associated locale.
	 */
	getHttpLocale(): string

	/**
	 * Returns HTTP method name (GET, POST, PUT, etc.).
	 */
	getHttpMethod(): string

	/**
	 * Returns parameter map containing HTTP parameters for current request.
	 */
	getHttpParameterMap(): HttpParameterMap

	/**
	 * Returns Map containing raw HTTP parameters sent to server.
	 */
	getHttpParameters(): Map

	/**
	 * Returns the path.
	 */
	getHttpPath(): string

	/**
	 * Returns HTTP protocol ("http" or "https"). Returns null if not HTTP request.
	 */
	getHttpProtocol(): string

	/**
	 * Returns query string or null if no query string.
	 */
	getHttpQueryString(): string

	/**
	 * Returns referer or null if no referer.
	 */
	getHttpReferer(): string

	/**
	 * Returns remote address or null if not found.
	 */
	getHttpRemoteAddress(): string

	/**
	 * Returns complete URL received at server.
	 */
	getHttpURL(): URL

	/**
	 * Returns HTTP user agent or null if no user agent.
	 */
	getHttpUserAgent(): string

	/**
	 * Returns locale of current request.
	 */
	getLocale(): string

	/**
	 * Returns OCAPI version of current request.
	 */
	getOcapiVersion(): string

	/**
	 * Returns page meta data associated with current request.
	 */
	getPageMetaData(): PageMetaData

	/**
	 * Returns unique identifier of current request.
	 */
	getRequestID(): string

	/**
	 * Returns map of SCAPI path parameters. Returns null if not SCAPI request.
	 */
	getSCAPIPathParameters(): Map

	/**
	 * Returns SCAPI path pattern. Returns null if not SCAPI request.
	 */
	getSCAPIPathPattern(): string

	/**
	 * Returns session associated with this request.
	 */
	getSession(): Session

	/**
	 * Returns form submitted by client if request represents form submission.
	 */
	getTriggeredForm(): Form

	/**
	 * Returns form action triggered by client if request represents form submission.
	 */
	getTriggeredFormAction(): FormAction

	/**
	 * Identifies if this is an HTTP request.
	 * @deprecated Effectively always returns true.
	 */
	isHttpRequest(): boolean

	/**
	 * Returns whether HTTP communication is secure (https).
	 */
	isHttpSecure(): boolean

	/**
	 * Returns true if request is for remote include, false if top-level request.
	 */
	isIncludeRequest(): boolean

	/**
	 * Returns whether request originated in SCAPI.
	 */
	isSCAPI(): boolean

	/**
	 * Sets physical location for current request. Value persists for session duration.
	 * @param geoLocation - the geolocation object to use
	 */
	setGeolocation(geoLocation: Geolocation): void

	/**
	 * Sets locale for request. Locale is set only if valid, active, and allowed for current site.
	 * @param localeID - the locale ID to be set, like 'en_US'
	 * @returns true if locale was successfully set, false otherwise
	 */
	setLocale(localeID: string): boolean
}
```
