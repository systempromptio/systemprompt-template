# dw.util.DateUtils

## Overview
Utility class providing static methods for working with Date objects in different time zones. This class is deprecated - use Calendar methods instead.

## Description
A class with several utility methods for Date objects.

**Deprecated:** See each method for additional information.

```
Object
  dw.util.DateUtils
```

```ts
declare class DateUtils  {
	/**
	 * Returns the current time stamp in the time zone of the instance.
	 * @deprecated Use System.getCalendar() instead.
	 * @returns the current time stamp in the time zone of the instance
	 */
	static nowForInstance(): Date

	/**
	 * Returns the current timestamp in the time zone of the current site.
	 * @deprecated Use Site.getCalendar() instead.
	 * @returns the current timestamp in the time zone of the current site
	 */
	static nowForSite(): Date

	/**
	 * Returns the current time stamp in UTC.
	 * @deprecated Create a new Calendar object and set the time zone "UTC" instead.
	 * @returns the current time stamp in UTC
	 */
	static nowInUTC(): Date
}
```
