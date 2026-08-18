# dw.web.Cookie

## Overview
Represents an HTTP cookie used for storing information on a client browser.

## Description
Represents an HTTP cookie used for storing information on a client browser. Cookies are passed along in the HTTP request and can be retrieved by calling `dw.system.Request.getHttpCookies()`. Cookies must comply with RFC6265. Use only printable ASCII characters without separators, such as a comma or equal sign. If JSON is used as a cookie value, it must be encoded. This class allows access to sensitive security-related data. Pay special attention to PCI DSS v3 requirements 2, 4, and 12.

```ts
declare class Cookie  {
	/**
	 * Default name for cookies with empty strings.
	 */
	static EMPTYNAME: 'dw_emptyname__'

	/**
	 * The comment associated with the cookie.
	 */
	comment: string

	/**
	 * The domain associated with the cookie.
	 */
	domain: string

	/**
	 * Identifies if the cookie is http-only.
	 */
	httpOnly: boolean

	/**
	 * The maximum age of the cookie in seconds. By default, -1 indicating the cookie will persist until client shutdown.
	 */
	maxAge: number

	/**
	 * The cookie's name.
	 */
	readonly name: string

	/**
	 * The path for the cookie.
	 */
	path: string

	/**
	 * Identifies if the cookie is secure.
	 */
	secure: boolean

	/**
	 * The cookie's value.
	 */
	value: string

	/**
	 * The version for the cookie. 0 means original Netscape cookie and 1 means RFC 2109 compliant cookie.
	 */
	version: number

	/**
	 * Constructs a new cookie using the specified name and value.
	 * @param name - the name for the cookie
	 * @param value - the cookie's value
	 */
	constructor(name: string, value: string)

	/**
	 * Returns the comment associated with the cookie.
	 */
	getComment(): string

	/**
	 * Returns the domain associated with the cookie.
	 */
	getDomain(): string

	/**
	 * Returns the maximum age of the cookie in seconds. By default, -1 indicating the cookie will persist until client shutdown.
	 */
	getMaxAge(): number

	/**
	 * Returns the cookie's name.
	 */
	getName(): string

	/**
	 * Returns the path for the cookie.
	 */
	getPath(): string

	/**
	 * Returns true if the cookie is secure, false otherwise.
	 */
	getSecure(): boolean

	/**
	 * Returns the cookie's value.
	 */
	getValue(): string

	/**
	 * Returns the version for the cookie. 0 means original Netscape cookie and 1 means RFC 2109 compliant cookie.
	 */
	getVersion(): number

	/**
	 * Returns true if the cookie is http-only, false otherwise.
	 */
	isHttpOnly(): boolean

	/**
	 * Sets the comment associated with the cookie. Setting a comment automatically changes the cookie to be a RFC 2109 (set-cookie2) compliant cookie.
	 * @param comment - the comment associated with the cookie
	 */
	setComment(comment: string): void

	/**
	 * Sets the domain associated with the cookie.
	 * @param domain - the domain associated with the cookie
	 */
	setDomain(domain: string): void

	/**
	 * Sets the http-only state for the cookie.
	 * @param httpOnly - sets http-only state for the cookie
	 */
	setHttpOnly(httpOnly: boolean): void

	/**
	 * Sets the maximum age of the cookie in seconds. A positive value indicates the cookie will expire after that many seconds. A negative value means the cookie is not stored persistently and will be deleted when the client exits. A zero value causes the cookie to be deleted.
	 * @param age - an integer specifying the maximum age in seconds; if negative, means not stored; if zero, deletes the cookie
	 */
	setMaxAge(age: number): void

	/**
	 * Sets the path for the cookie.
	 * @param path - the path for the cookie
	 */
	setPath(path: string): void

	/**
	 * Sets the secure state for the cookie.
	 * @param secure - sets secure state for the cookie
	 */
	setSecure(secure: boolean): void

	/**
	 * Sets the cookie's value.
	 * @param value - the value to set in the cookie
	 */
	setValue(value: string): void

	/**
	 * Sets the version for the cookie. 0 means original Netscape cookie and 1 means RFC 2109 compliant cookie. The default is 0.
	 * @param version - the version for the cookie
	 */
	setVersion(version: number): void
}
```
