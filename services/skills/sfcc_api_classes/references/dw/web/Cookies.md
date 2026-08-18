# dw.web.Cookies

## Overview
Provides index and associative array-like access to HTTP cookies from the current request. Allows secure, read-only access to cookie metadata.

## Description
The class provides an index and associative array like access to the Cookies of the current request. Cookies can be retrieved by calling `dw.system.Request.getHttpCookies()`.

**Note:** this class allows access to sensitive security-related data. Pay special attention to PCI DSS v3. requirements 2, 4, and 12.

```ts
declare class Cookies  {
	/**
	 * The number of known cookies.
	 */
	readonly cookieCount: number

	/**
	 * Returns the number of known cookies.
	 */
	getCookieCount(): number
}
```
