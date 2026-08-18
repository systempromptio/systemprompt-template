# dw.web.URLAction

## Overview
Represents a reference to a pipeline/controller name and start node for URL creation within template processing.

## Description
Used in HREF or FORM action attributes. Instances are passed to URLUtils methods to generate properly constructed Commerce Cloud Digital URLs. Supports site, locale, and hostname configuration for URL generation.

```ts
declare class URLAction {
	/**
	 * Constructs an action for the current site and locale.
	 * @param action - The target pipeline/controller, e.g. 'Default-Start'
	 */
	constructor(action: string)

	/**
	 * Constructs an action for the specified site and the current locale.
	 * @param action - The target pipeline/controller, e.g. 'Default-Start'
	 * @param siteName - The target site, e.g. 'SampleSite'
	 */
	constructor(action: string, siteName: string)

	/**
	 * Constructs an action for the specified site and locale.
	 * @param action - The target pipeline/controller, e.g. 'Default-Start'
	 * @param siteName - The target site, e.g. 'SampleSite'
	 * @param locale - The target locale, e.g. 'default'
	 */
	constructor(action: string, siteName: string, locale: string)

	/**
	 * Constructs an URL action for the specified site, locale and hostname. The hostname must be defined in the site alias settings. If no hostname is provided, the HTTP/HTTPS host defined in the site alias settings will be used. If no HTTP/HTTPS host is defined, the hostname of the current request is used.
	 * @param action - The target pipeline/controller, e.g. 'Default-Start'
	 * @param siteName - The target site, e.g. 'SampleSite'
	 * @param locale - The target locale, e.g. 'default'
	 * @param hostName - The host name, e.g. 'www.shop.com'
	 * @throws Exception if hostName is not defined in site alias settings
	 */
	constructor(action: string, siteName: string, locale: string, hostName: string)
}
```
