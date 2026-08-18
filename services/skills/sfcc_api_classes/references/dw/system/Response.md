# dw.system.Response

## Overview
Represents an HTTP response in Commerce Cloud Digital. Implicitly available as global `response` variable.

## Description
Use to set cookies, HTTP headers, access output stream, or send redirects. For public headers, only names listed in Constants are allowed. Custom headers must begin with "X-SF-CC-" and contain only alphanumeric, dash, and underscore characters.

```ts
declare class Response  {
	/**
	 * Header name constant for Access-Control-Allow-Credentials.
	 */
	static ACCESS_CONTROL_ALLOW_CREDENTIALS: 'Access-Control-Allow-Credentials'

	/**
	 * Header name constant for Access-Control-Allow-Headers.
	 */
	static ACCESS_CONTROL_ALLOW_HEADERS: 'Access-Control-Allow-Headers'

	/**
	 * Header name constant for Access-Control-Allow-Methods.
	 */
	static ACCESS_CONTROL_ALLOW_METHODS: 'Access-Control-Allow-Methods'

	/**
	 * Header name constant for Access-Control-Allow-Origin.
	 */
	static ACCESS_CONTROL_ALLOW_ORIGIN: 'Access-Control-Allow-Origin'

	/**
	 * Header name constant for Access-Control-Expose-Headers.
	 */
	static ACCESS_CONTROL_EXPOSE_HEADERS: 'Access-Control-Expose-Headers'

	/**
	 * Header name constant for Allow.
	 */
	static ALLOW: 'Allow'

	/**
	 * Header name constant for Content-Disposition.
	 */
	static CONTENT_DISPOSITION: 'Content-Disposition'

	/**
	 * Header name constant for Content-Language.
	 */
	static CONTENT_LANGUAGE: 'Content-Language'

	/**
	 * Header name constant for Content-Location.
	 */
	static CONTENT_LOCATION: 'Content-Location'

	/**
	 * Header name constant for Content-MD5.
	 */
	static CONTENT_MD5: 'Content-MD5'

	/**
	 * Header name constant for Content-Security-Policy. Platform can override this header for tools like Storefront Toolkit.
	 */
	static CONTENT_SECURITY_POLICY: 'Content-Security-Policy'

	/**
	 * Header name constant for Content-Security-Policy-Report-Only. Storefront requests only. Report recipient can't be B2C Commerce system.
	 */
	static CONTENT_SECURITY_POLICY_REPORT_ONLY: 'Content-Security-Policy-Report-Only'

	/**
	 * Header name constant for Content-Type.
	 */
	static CONTENT_TYPE: 'Content-Type'

	/**
	 * Header name constant for Cross-Origin-Embedder-Policy.
	 */
	static CROSS_ORIGIN_EMBEDDER_POLICY: 'Cross-Origin-Embedder-Policy'

	/**
	 * Header name constant for Cross-Origin-Embedder-Policy-Report-Only. Storefront requests only. Report recipient can't be B2C Commerce system.
	 */
	static CROSS_ORIGIN_EMBEDDER_POLICY_REPORT_ONLY: 'Cross-Origin-Embedder-Policy-Report-Only'

	/**
	 * Header name constant for Cross-Origin-Opener-Policy.
	 */
	static CROSS_ORIGIN_OPENER_POLICY: 'Cross-Origin-Opener-Policy'

	/**
	 * Header name constant for Cross-Origin-Opener-Policy-Report-Only. Storefront requests only. Report recipient can't be B2C Commerce system.
	 */
	static CROSS_ORIGIN_OPENER_POLICY_REPORT_ONLY: 'Cross-Origin-Opener-Policy-Report-Only'

	/**
	 * Header name constant for Cross-Origin-Resource-Policy.
	 */
	static CROSS_ORIGIN_RESOURCE_POLICY: 'Cross-Origin-Resource-Policy'

	/**
	 * Header name constant for Link.
	 */
	static LINK: 'Link'

	/**
	 * Header name constant for Location.
	 */
	static LOCATION: 'Location'

	/**
	 * Header name constant for Permissions-Policy.
	 */
	static PERMISSIONS_POLICY: 'Permissions-Policy'

	/**
	 * Header name constant for Platform for Privacy Preferences Project.
	 */
	static PLATFORM_FOR_PRIVACY_PREFERENCES_PROJECT: 'P3P'

	/**
	 * Header name constant for Referrer-Policy.
	 */
	static REFERRER_POLICY: 'Referrer-Policy'

	/**
	 * Header name constant for Refresh.
	 */
	static REFRESH: 'Refresh'

	/**
	 * Header name constant for Retry-After.
	 */
	static RETRY_AFTER: 'Retry-After'

	/**
	 * Header name constant for service-worker-allowed.
	 */
	static SERVICE_WORKER_ALLOWED: 'service-worker-allowed'

	/**
	 * Header name constant for Vary.
	 */
	static VARY: 'Vary'

	/**
	 * Header name constant for X-Content-Type-Options.
	 */
	static X_CONTENT_TYPE_OPTIONS: 'X-Content-Type-Options'

	/**
	 * Header name constant for X-FRAME-OPTIONS. Platform can override this header for tools like Storefront Toolkit.
	 */
	static X_FRAME_OPTIONS: 'X-FRAME-OPTIONS'

	/**
	 * Allowed value ALLOW-FROM for X-FRAME-OPTIONS.
	 */
	static X_FRAME_OPTIONS_ALLOW_FROM: 'ALLOW-FROM'

	/**
	 * Allowed value DENY for X-FRAME-OPTIONS.
	 */
	static X_FRAME_OPTIONS_DENY_VALUE: 'DENY'

	/**
	 * Allowed value SAMEORIGIN for X-FRAME-OPTIONS.
	 */
	static X_FRAME_OPTIONS_SAMEORIGIN_VALUE: 'SAMEORIGIN'

	/**
	 * Header name constant for X-Robots-Tag.
	 */
	static X_ROBOTS_TAG: 'X-Robots-Tag'

	/**
	 * Header name constant for X-XSS-Protection.
	 */
	static X_XSS_PROTECTION: 'X-XSS-Protection'

	/**
	 * PrintWriter for printing content directly to response.
	 * @readonly
	 */
	readonly writer: PrintWriter

	/**
	 * Adds cookie to outgoing response. Can be called multiple times. Last-set cookie with same name/domain/path wins. Set maxAge to 0 to delete cookie. SameSite attribute is set to None if Secure flag is set or Enforce HTTPS is enabled.
	 * @param cookie - a Cookie object
	 */
	addHttpCookie(cookie: Cookie): void

	/**
	 * Adds response header with given name and value. Allows multiple values.
	 * @param name - the name to use for the response header
	 * @param value - the value to use
	 */
	addHttpHeader(name: string, value: string): void

	/**
	 * Checks whether response message header has field with specified name.
	 * @param name - the name to use
	 */
	containsHttpHeader(name: string): boolean

	/**
	 * Returns PrintWriter for printing content directly to response.
	 */
	getWriter(): PrintWriter

	/**
	 * Sends temporary redirect (HTTP 302) to client for specified URL.
	 * @param url - the URL object for target location
	 */
	redirect(url: URL): void

	/**
	 * Sends redirect response with given status to client for specified URL.
	 * @param url - the URL object with redirect location
	 * @param status - status code (must be 301, 302 or 307)
	 */
	redirect(url: URL, status: number): void

	/**
	 * Sends temporary redirect (HTTP 302) to client for specified location. Target must be relative or absolute URL.
	 * @param location - target location as string
	 */
	redirect(location: string): void

	/**
	 * Sends redirect response with given status to client for specified location.
	 * @param location - redirect location
	 * @param status - status code (must be 301, 302 or 307)
	 */
	redirect(location: string, status: number): void

	/**
	 * Sends redirect response with given status to client for specified redirect.
	 * @param redirect - URLRedirect object with location and status
	 */
	redirect(redirect: URLRedirect): void

	/**
	 * Sets whether output should be buffered or streamed. By default buffering is enabled. Can only be changed before anything written to response. Streaming recommended for large responses.
	 * @param buffered - if true buffering is used, if false response is streamed
	 */
	setBuffered(buffered: boolean): void

	/**
	 * Sets content type for response. May only be called before any output written.
	 * @param contentType - MIME type (e.g., "text/html", "application/json")
	 */
	setContentType(contentType: string): void

	/**
	 * Sets cache expiration time for response. Response cached only if caching not disabled. By default responses not cached. If called multiple times, lowest expiration time wins. Only for HTTP requests. Streamed responses can't be cached.
	 * @param expires - expiration time in milliseconds since January 1, 1970, 00:00:00 GMT
	 */
	setExpires(expires: number): void

	/**
	 * Convenience method for setExpires(Number) which takes Date object.
	 * @param expires - a Date object
	 */
	setExpires(expires: Date): void

	/**
	 * Adds response header with given name and value. Overwrites previous values. Use containsHttpHeader() to test for presence before setting.
	 * @param name - the name to use for the response header
	 * @param value - the value to use
	 */
	setHttpHeader(name: string, value: string): void

	/**
	 * Sets HTTP response code.
	 * @param status - standard HTTP status code (e.g., 200 for "OK")
	 */
	setStatus(status: number): void

	/**
	 * Marks response as personalized with given variant identifier. Platform caches variants based on pricebook, promotion, sorting rule and A/B test segments. Once set, entire response treated as personalized. Equivalent to <iscache varyby="price_promotion" />.
	 * @param varyBy - variation criteria (currently only "price_promotion" is supported)
	 */
	setVaryBy(varyBy: string): void
}
```
