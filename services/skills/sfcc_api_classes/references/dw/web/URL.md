# dw.web.URL

## Overview
Represents a URL in Commerce Cloud Digital with methods for URL manipulation and configuration.

## Description
Provides URL construction and modification capabilities including protocol switching, host configuration, parameter management, and CSRF token handling.

```ts
declare class URL {
	/**
	 * Makes the URL absolute and ensures that the protocol of the request is used or http in a mail context.
	 * @returns A new URL instance
	 * @throws RuntimeException if called on static content or image transformation URLs
	 */
	abs(): URL

	/**
	 * Append a request parameter to this URL.
	 * @param name - The parameter name. Must not be null
	 * @param value - The parameter value. If null, then treated as empty value
	 * @returns A reference to this URL
	 */
	append(name: string, value: string): URL

	/**
	 * Appends, if applicable, a CSRF protection token to this URL. The CSRF token will only be appended if the URL is a pipeline URL for Business Manager. If a CSRF token already exists, it will be replaced with a newly generated one.
	 * @returns A reference to this URL, with a CSRF token appended if applicable
	 */
	appendCSRFTokenBM(): URL

	/**
	 * Updates the URL with the specified host name.
	 * @param host - The host name that is used to update the URL
	 * @returns A new URL instance
	 * @throws RuntimeException if called on static content or image transformation URLs
	 */
	host(host: string): URL

	/**
	 * Makes the URL absolute and ensures that the protocol http is used.
	 * @returns A new URL instance
	 * @throws RuntimeException if called on static content or image transformation URLs
	 */
	http(): URL

	/**
	 * Makes the URL absolute and ensures that the protocol https is used.
	 * @returns A new URL instance
	 * @throws RuntimeException if called on static content or image transformation URLs
	 */
	https(): URL

	/**
	 * Makes the URL relative.
	 * @returns A new URL instance
	 * @throws RuntimeException if called on static content or image transformation URLs
	 */
	relative(): URL

	/**
	 * Remove a request parameter from this URL. If the parameter is not part of the URL, nothing is done.
	 * @param name - The parameter name. Must not be null
	 * @returns A reference to this URL
	 */
	remove(name: string): URL

	/**
	 * Updates the URL with the site host name.
	 * @returns A new URL instance
	 * @throws RuntimeException if called on static content or image transformation URLs
	 */
	siteHost(): URL

	/**
	 * Return String representation of the URL.
	 * @returns The URL as a string
	 */
	toString(): string
}
```
