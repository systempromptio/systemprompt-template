# dw.web.URLRedirect

## Overview
Represents a URLRedirect in Commerce Cloud Digital.

## Description
Contains redirect URL location and corresponding HTTP status code for URL redirects.

```ts
declare class URLRedirect {
	/**
	 * The URL which was calculated to be the redirect URL. The Location parameter can be directly used as value for a redirect location.
	 */
	readonly location: string

	/**
	 * The corresponding status code for the redirect location.
	 */
	readonly status: number

	/**
	 * Returns the URL which was calculated to be the redirect URL. The Location parameter can be directly used as value for a redirect location.
	 * @returns Redirect location
	 */
	getLocation(): string

	/**
	 * Returns the corresponding status code for the redirect location.
	 * @returns Status code
	 */
	getStatus(): number
}
```
