# dw.util.Locale

## Overview
Represents a Locale supported by the system with language, country, and display name information.

## Description
Represents system-supported Locales with ISO 639 language codes and ISO 3166 country codes. Provides access to display names in the Locale's own language. Primary key is the localeID combining language and country (e.g., "en_US").

```ts
declare class Locale  {
	/**
	 * The uppercase ISO 3166 2-letter country/region code
	 * Returns empty string if no country specified
	 */
	readonly country: String

	/**
	 * Display name of this Locale's country in this Locale's language
	 * Returns empty string if no country specified
	 */
	readonly displayCountry: String

	/**
	 * Display name of this Locale's language in this Locale's language
	 * Returns empty string if no language specified
	 */
	readonly displayLanguage: String

	/**
	 * Display name of this Locale in this Locale's language
	 * Returns empty string if no display name specified
	 */
	readonly displayName: String

	/**
	 * String representation of the localeID
	 * Combines language and country key concatenated with underscore (e.g., "en_US")
	 * Primary key of the class
	 */
	readonly ID: String

	/**
	 * The uppercase ISO 3166 3-letter country/region code
	 * Returns empty string if no country specified
	 */
	readonly ISO3Country: String

	/**
	 * The 3-letter ISO 639 language code
	 * Returns empty string if no language specified
	 */
	readonly ISO3Language: String

	/**
	 * The lowercase ISO 639 language code
	 * Returns empty string if no language specified
	 */
	readonly language: String

	/**
	 * Returns the uppercase ISO 3166 2-letter country/region code
	 */
	getCountry(): String

	/**
	 * Returns the display name of this Locale's country in this Locale's language
	 */
	getDisplayCountry(): String

	/**
	 * Returns the display name of this Locale's language in this Locale's language
	 */
	getDisplayLanguage(): String

	/**
	 * Returns the display name of this Locale in this Locale's language
	 */
	getDisplayName(): String

	/**
	 * Returns the String representation of the localeID
	 * Combines language and country key concatenated with underscore (e.g., "en_US")
	 */
	getID(): String

	/**
	 * Returns the uppercase ISO 3166 3-letter country/region code
	 */
	getISO3Country(): String

	/**
	 * Returns the 3-letter ISO 639 language code
	 */
	getISO3Language(): String

	/**
	 * Returns the lowercase ISO 639 language code
	 */
	getLanguage(): String

	/**
	 * Returns a Locale instance for the given localeId or null if not found
	 */
	static getLocale(localeId: String): Locale

	/**
	 * Returns the String representation of the localeID
	 */
	toString(): String
}
```
