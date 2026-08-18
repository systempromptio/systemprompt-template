# dw.util.Geolocation

## Overview
Read-only class representing a geographic position with latitude/longitude coordinates and associated location metadata (country, city, region, etc.).

## Description
Represents a position on earth (latitude and longitude) and information associated with that location (e.g. country, city, etc). Commerce Cloud can provide geolocation information for a Request, useful in customer group segmentation rules. Not related to store locator API (GetNearestStores pipelet) which uses static merchant-loaded store locations. Uses GeoLite2 data created by MaxMind.

```
Object
  dw.util.Geolocation
```

```ts
declare class Geolocation  {
	/**
	 * Returns true if a valid GeoLocation was found for the IP address (meaning at least Latitude and Longitude were found).
	 */
	readonly available: boolean

	/**
	 * City name in English associated with this location.
	 */
	readonly city: string

	/**
	 * ISO country code associated with this location.
	 */
	readonly countryCode: string

	/**
	 * Country name in English associated with this location.
	 */
	readonly countryName: string

	/**
	 * Latitude coordinate associated with this location (number between -90.0 and +90.0).
	 */
	readonly latitude: Number

	/**
	 * Longitude coordinate associated with this location (number between -180.0 and +180.0).
	 */
	readonly longitude: Number

	/**
	 * Metro code associated with this location (US locations only).
	 */
	readonly metroCode: string

	/**
	 * Postal code associated with this location.
	 */
	readonly postalCode: string

	/**
	 * Region (e.g. province or state) code for this location.
	 */
	readonly regionCode: string

	/**
	 * Region (e.g. province or state) name in English associated with this location.
	 */
	readonly regionName: string

	/**
	 * Constructor for a Geolocation object.
	 * @param countryCode - ISO 3166-1 alpha-2 country code
	 * @param countryName - Country name in English
	 * @param regionCode - Region code (up to 3 characters, ISO 3166-2 subdivision portion)
	 * @param regionName - Region name in English
	 * @param metroCode - Metro code (US locations only, see Google AdWords API for values)
	 * @param city - City name in English
	 * @param postalCode - Postal code
	 * @param latitude - Latitude coordinate (-90.0 to +90.0)
	 * @param longitude - Longitude coordinate (-180.0 to +180.0)
	 */
	constructor(countryCode: string, countryName: string, regionCode: string, regionName: string, metroCode: string, city: string, postalCode: string, latitude: Number, longitude: Number)

	/**
	 * Get the city name in English associated with this location.
	 * @returns the city name
	 */
	getCity(): string

	/**
	 * Get the ISO country code associated with this location.
	 * @returns the two-character ISO 3166-1 alpha code for the country
	 */
	getCountryCode(): string

	/**
	 * Get the country name in English associated with this location.
	 * @returns the country name
	 */
	getCountryName(): string

	/**
	 * Get the latitude coordinate associated with this location (number between -90.0 and +90.0).
	 * @returns the latitude as a floating point number
	 */
	getLatitude(): Number

	/**
	 * Get the longitude coordinate associated with this location (number between -180.0 and +180.0).
	 * @returns the longitude as a floating point number
	 */
	getLongitude(): Number

	/**
	 * Get the metro code associated with this location.
	 * @returns the metro code (US locations only, see Google AdWords API for values)
	 */
	getMetroCode(): string

	/**
	 * Get the postal code associated with this location.
	 * @returns the postal code (not available for all countries; may contain only part of the postal code)
	 */
	getPostalCode(): string

	/**
	 * Get the region (e.g. province or state) code for this location.
	 * @returns string up to 3 characters containing the ISO 3166-2 subdivision code
	 */
	getRegionCode(): string

	/**
	 * Get the region (e.g. province or state) name in English associated with this location.
	 * @returns the region name
	 */
	getRegionName(): string

	/**
	 * Returns true if a valid GeoLocation was found for the IP address (at least Latitude and Longitude).
	 * @returns true if valid geolocation available, false otherwise
	 */
	isAvailable(): boolean
}
```
