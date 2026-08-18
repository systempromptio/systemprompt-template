# dw.system.Site

## Overview
Represents a site in Commerce Cloud Digital, providing access to site-level configuration values managed from Business Manager.

## Description
This class represents a site in Commerce Cloud Digital and provides access to several site-level configuration values managed from Business Manager. Only possible to get reference to current site as determined by current request. The static method getCurrent() returns reference to current site.

```ts
declare class Site  {
  /**
   * Constant representing Site under maintenance/offline.
   */
  static SITE_STATUS_MAINTENANCE: 3

  /**
   * Constant representing Site is Online.
   */
  static SITE_STATUS_ONLINE: 1

  /**
   * Constant representing Site is in preview mode or online/password (protected).
   */
  static SITE_STATUS_PROTECTED: 5

  /**
   * Allowed currencies of current site as collection of currency codes.
   */
  readonly allowedCurrencies: List

  /**
   * Allowed locales of current site as collection of locale IDs.
   */
  readonly allowedLocales: List

  /**
   * All sites.
   */
  readonly allSites: List

  /**
   * New Calendar object in time zone of current site.
   */
  readonly calendar: Calendar

  /**
   * Default currency code for current site.
   * @deprecated Use getDefaultCurrency() method instead
   */
  readonly currencyCode: string

  /**
   * The current site.
   */
  readonly current: Site

  /**
   * Default currency code for current site.
   */
  readonly defaultCurrency: string

  /**
   * Default locale for the site.
   */
  readonly defaultLocale: string

  /**
   * Einstein site Id. Typically concatenation of realm, underscore and site id. Can be overwritten by support for realm moves. Used when calling Einstein APIs.
   */
  readonly einsteinSiteID: string

  /**
   * Configured HTTP host name. Returns instance hostname if not configured.
   */
  readonly httpHostName: string

  /**
   * Configured HTTPS host name. Returns HTTP host name or instance hostname if not configured.
   */
  readonly httpsHostName: string

  /**
   * ID of the site.
   */
  readonly ID: string

  /**
   * Descriptive name for the site.
   */
  readonly name: string

  /**
   * Whether OMS is active in current site.
   * @deprecated This item is deprecated
   */
  readonly OMSEnabled: boolean

  /**
   * All page meta tags defined for this instance for which content can be generated. Content generated based on home page meta tag context and rules from current repository domain.
   */
  readonly pageMetaTags: Array

  /**
   * Container of all site preferences of this site.
   */
  readonly preferences: SitePreferences

  /**
   * Status of this site. Possible values: SITE_STATUS_ONLINE, SITE_STATUS_MAINTENANCE, SITE_STATUS_PROTECTED.
   */
  readonly status: number

  /**
   * Code for time zone in which storefront is running.
   */
  readonly timezone: string

  /**
   * Time zone offset in which storefront is running.
   */
  readonly timezoneOffset: number

  /**
   * Returns allowed currencies of current site as collection of currency codes.
   */
  getAllowedCurrencies(): List

  /**
   * Returns allowed locales of current site as collection of locale IDs.
   */
  getAllowedLocales(): List

  /**
   * Returns all sites.
   */
  static getAllSites(): List

  /**
   * Returns new Calendar object in time zone of current site.
   */
  static getCalendar(): Calendar

  /**
   * Returns default currency code for current site.
   * @deprecated Use getDefaultCurrency() method instead
   */
  getCurrencyCode(): string

  /**
   * Returns current site.
   */
  static getCurrent(): Site

  /**
   * Returns custom preference value. Returns null if preference does not exist. Shortcut for accessing custom attribute on SitePreferences object.
   * @param name - Preference name
   * @returns Preference value, or null if no preference with given name
   */
  getCustomPreferenceValue(name: string): Object

  /**
   * Returns default currency code for current site.
   */
  getDefaultCurrency(): string

  /**
   * Return default locale for the site.
   */
  getDefaultLocale(): string

  /**
   * Returns Einstein site Id. Typically concatenation of realm, underscore and site id. Can be overwritten by support for realm moves. Used when calling Einstein APIs.
   */
  getEinsteinSiteID(): string

  /**
   * Returns configured HTTP host name. Returns instance hostname if not configured.
   */
  getHttpHostName(): string

  /**
   * Returns configured HTTPS host name. Returns HTTP host name or instance hostname if not configured.
   */
  getHttpsHostName(): string

  /**
   * Returns ID of the site.
   */
  getID(): string

  /**
   * Returns descriptive name for the site.
   */
  getName(): string

  /**
   * Returns page meta tag for specified id. Content generated based on home page meta tag context and rule from current repository domain. Returns null if meta tag undefined on current instance, or if no rule found for current context, or if rule resolves to empty string.
   * @param id - ID to get page meta tag for
   * @returns Page meta tag containing content generated based on rules
   */
  getPageMetaTag(id: string): PageMetaTag

  /**
   * Returns all page meta tags defined for this instance for which content can be generated. Content generated based on home page meta tag context and rules from current repository domain.
   */
  getPageMetaTags(): Array

  /**
   * Returns container of all site preferences of this site.
   */
  getPreferences(): SitePreferences

  /**
   * Returns status of this site. Possible values: SITE_STATUS_ONLINE, SITE_STATUS_MAINTENANCE, SITE_STATUS_PROTECTED.
   */
  getStatus(): number

  /**
   * Returns code for time zone in which storefront is running.
   */
  getTimezone(): string

  /**
   * Returns time zone offset in which storefront is running.
   */
  getTimezoneOffset(): number

  /**
   * Whether OMS is active in current site.
   * @deprecated This item is deprecated
   */
  isOMSEnabled(): boolean

  /**
   * Sets value for custom preference. Value type must match declared type of preference definition.
   * @param name - Name of preference
   * @param value - New value for preference
   */
  setCustomPreferenceValue(name: string, value: Object): void
}
```
