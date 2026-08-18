# dw.web.URLRedirectMgr

## Overview
Manager class for generating URL redirects based on Static, Dynamic and URLRedirect mappings configured in Commerce Cloud Digital.

## Description
URLRedirect manager class. Methods in this class generate URLRedirects based on the current configuration for Static, Dynamic and URLRedirect mappings in Commerce Cloud Digital.

Information used to calculate URLRedirects are determined from the current HTTP request. The URL which is used to find a redirect can be accessed with getRedirectOrigin().

```
Object
  dw.web.URLRedirectMgr
```

```ts
declare class URLRedirectMgr {
	/**
	 * An URLRedirect object, containing a location and status. The redirect is calculated based on origin url of current request and the configured Static, Dynamic and URLRedirect mappings for the requested site.
	 */
	readonly redirect: URLRedirect

	/**
	 * The relative origin url (without protocol, port, hostname and site path information) which will be used in getRedirect() to calculate a redirect location for.
	 */
	readonly redirectOrigin: string

	/**
	 * Returns an URLRedirect object, containing a location and status. The redirect is calculated based on origin url of current request and the configured Static, Dynamic and URLRedirect mappings for the requested site.
	 * @returns URLRedirect containing the location and status code, null in case of no redirect was found
	 */
	static getRedirect(): URLRedirect

	/**
	 * Returns the relative origin url (without protocol, port, hostname and site path information) which will be used in getRedirect() to calculate a redirect location for.
	 * @returns relative origin url
	 */
	static getRedirectOrigin(): string
}
```
