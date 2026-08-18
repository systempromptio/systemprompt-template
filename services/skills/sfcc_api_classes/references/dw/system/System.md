# dw.system.System

## Overview
Represents the Commerce Cloud Digital server instance with information about instance type, timezone, and global preferences.

## Description
Represents the Commerce Cloud Digital server instance. An application server instance is configured to be one of three types: development system, staging system, or production system.

```ts
declare class System  {
	/**
	 * Represents the development system.
	 */
	static DEVELOPMENT_SYSTEM: 0

	/**
	 * Represents the production system.
	 */
	static PRODUCTION_SYSTEM: 2

	/**
	 * Represents the staging system.
	 */
	static STAGING_SYSTEM: 1

	/**
	 * A new Calendar object in the time zone of the current instance.
	 */
	readonly calendar: Calendar

	/**
	 * The compatibility mode of the custom code version that is currently active.
	 * Returned as a number, e.g. compatibility mode "15.5" is returned as 1505.
	 */
	readonly compatibilityMode: number

	/**
	 * Returns instance hostname.
	 */
	readonly instanceHostname: string

	/**
	 * The instance time zone. The time zone in which global actions like jobs or reporting are specified.
	 * Note: the instance time zone is cached at the current session. Changes affect only new sessions.
	 */
	readonly instanceTimeZone: string

	/**
	 * The type of the instance. One of: development system, staging system, or production system.
	 */
	readonly instanceType: number

	/**
	 * A container of all global preferences of this organization (instance).
	 */
	readonly preferences: OrganizationPreferences

	/**
	 * Returns a new Calendar object in the time zone of the current instance.
	 * @returns a Calendar object in the time zone of the instance
	 */
	static getCalendar(): Calendar

	/**
	 * Returns the compatibility mode of the custom code version that is currently active.
	 * Returned as a number, e.g. compatibility mode "15.5" is returned as 1505.
	 * @returns the currently active compatibility mode
	 */
	static getCompatibilityMode(): number

	/**
	 * Returns instance hostname.
	 * @returns instance hostname
	 */
	static getInstanceHostname(): string

	/**
	 * Returns the instance time zone. The time zone in which global actions like jobs or reporting are specified.
	 * Note: the instance time zone is cached at the current session. Changes affect only new sessions.
	 * @returns the instance time zone
	 */
	static getInstanceTimeZone(): string

	/**
	 * Returns the type of the instance. One of: DEVELOPMENT_SYSTEM, STAGING_SYSTEM, or PRODUCTION_SYSTEM.
	 * @returns the instance type of the application server where this method was called
	 */
	static getInstanceType(): number

	/**
	 * Returns a container of all global preferences of this organization (instance).
	 * @returns a preferences object containing all global system and custom preferences of this instance
	 */
	static getPreferences(): OrganizationPreferences
}
```
