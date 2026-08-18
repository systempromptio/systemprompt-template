# dw.util.Calendar

## Overview
Represents a Calendar based on java.util.Calendar. Provides date/time manipulation, field operations, and locale-aware formatting.

## Description
Represents a Calendar and is based on the java.util.Calendar class. Use StringUtils.formatCalendar(Calendar) functions to convert a Calendar object into a String.

```ts
declare class Calendar  {
	/**
	 * Indicates whether the HOUR is before or after noon.
	 */
	static AM_PM: 9

	/**
	 * Value for the month of year field representing April.
	 */
	static APRIL: 3

	/**
	 * Value for the month of year field representing August.
	 */
	static AUGUST: 7

	/**
	 * Represents a date.
	 */
	static DATE: 5

	/**
	 * Represents a day of the month.
	 */
	static DAY_OF_MONTH: 5

	/**
	 * Represents a day of the week.
	 */
	static DAY_OF_WEEK: 7

	/**
	 * Represents a day of the week in a month.
	 */
	static DAY_OF_WEEK_IN_MONTH: 8

	/**
	 * Represents a day of the year.
	 */
	static DAY_OF_YEAR: 6

	/**
	 * Value for the month of year field representing December.
	 */
	static DECEMBER: 11

	/**
	 * Indicates the daylight savings offset in milliseconds.
	 */
	static DST_OFFSET: 16

	/**
	 * Indicates the era such as 'AD' or 'BC' in the Julian calendar.
	 */
	static ERA: 0

	/**
	 * Value for the month of year field representing February.
	 */
	static FEBRUARY: 1

	/**
	 * Value for the day of the week field representing Friday.
	 */
	static FRIDAY: 6

	/**
	 * Represents an hour.
	 */
	static HOUR: 10

	/**
	 * Represents an hour of the day.
	 */
	static HOUR_OF_DAY: 11

	/**
	 * The input date pattern, for instance MM/dd/yyyy
	 */
	static INPUT_DATE_PATTERN: 3

	/**
	 * The input date time pattern, for instance MM/dd/yyyy h:mm a
	 */
	static INPUT_DATE_TIME_PATTERN: 5

	/**
	 * The input time pattern, for instance h:mm a
	 */
	static INPUT_TIME_PATTERN: 4

	/**
	 * Value for the month of year field representing January.
	 */
	static JANUARY: 0

	/**
	 * Value for the month of year field representing July.
	 */
	static JULY: 6

	/**
	 * Value for the month of year field representing June.
	 */
	static JUNE: 5

	/**
	 * The long date pattern, for instance MMM/d/yyyy
	 */
	static LONG_DATE_PATTERN: 1

	/**
	 * Value for the month of year field representing March.
	 */
	static MARCH: 2

	/**
	 * Value for the month of year field representing May.
	 */
	static MAY: 4

	/**
	 * Represents a millisecond.
	 */
	static MILLISECOND: 14

	/**
	 * Represents a minute.
	 */
	static MINUTE: 12

	/**
	 * Value for the day of the week field representing Monday.
	 */
	static MONDAY: 2

	/**
	 * Represents a month where the first month of the year is 0.
	 */
	static MONTH: 2

	/**
	 * Value for the month of year field representing November.
	 */
	static NOVEMBER: 10

	/**
	 * Value for the month of year field representing October.
	 */
	static OCTOBER: 9

	/**
	 * Value for the day of the week field representing Saturday.
	 */
	static SATURDAY: 7

	/**
	 * Represents a second.
	 */
	static SECOND: 13

	/**
	 * Value for the month of year field representing September.
	 */
	static SEPTEMBER: 8

	/**
	 * The short date pattern, for instance M/d/yy
	 */
	static SHORT_DATE_PATTERN: 0

	/**
	 * Value for the day of the week field representing Sunday.
	 */
	static SUNDAY: 1

	/**
	 * Value for the day of the week field representing Thursday.
	 */
	static THURSDAY: 5

	/**
	 * The time pattern, for instance h:mm:ss a
	 */
	static TIME_PATTERN: 2

	/**
	 * Value for the day of the week field representing Tuesday.
	 */
	static TUESDAY: 3

	/**
	 * Value for the day of the week field representing Wednesday.
	 */
	static WEDNESDAY: 4

	/**
	 * Represents a week of the month.
	 */
	static WEEK_OF_MONTH: 4

	/**
	 * Represents a week in the year.
	 */
	static WEEK_OF_YEAR: 3

	/**
	 * Represents a year.
	 */
	static YEAR: 1

	/**
	 * Indicates the raw offset from GMT in milliseconds.
	 */
	static ZONE_OFFSET: 15

	/**
	 * The first day of the week based on locale context. In the US the first day of the week is SUNDAY. In France the first day of the week is MONDAY.
	 */
	readonly firstDayOfWeek: Number

	/**
	 * The current time stamp of this calendar. This method is also used to convert a Calendar into a Date. WARNING: The returned Date object's time is always interpreted in the time zone GMT. This means time zone information set at the calendar object will not be honored and gets lost.
	 */
	readonly time: Date

	/**
	 * The current time zone of this calendar.
	 */
	readonly timeZone: String

	/**
	 * Creates a new Calendar object that is set to the current time. The default time zone of the Calendar object is GMT. WARNING: Keep in mind that the time stamp represented by the new calendar is always interpreted in the time zone GMT. This means time zone information at the calendar object needs to be set separately by using the setTimeZone(String) method.
	 */
	constructor()

	/**
	 * Creates a new Calendar object for the given Date object. The time is set to the given Date object's time. The default time zone of the Calendar object is GMT. WARNING: Keep in mind that the given Date object is always interpreted in the time zone GMT. This means time zone information at the calendar object needs to be set separately by using the setTimeZone(String) method.
	 * @param date - the date for which the calendar will be set
	 */
	constructor(date: Date)

	/**
	 * Adds or subtracts the specified amount of time to the given calendar field, based on the calendar's rules.
	 * @param field - the calendar field
	 * @param value - the amount of date or time to be added to the field
	 */
	add(field: Number, value: Number): void

	/**
	 * Indicates if this Calendar represents a time after the time represented by the specified Object.
	 * @param obj - the object to test
	 * @returns true if this Calendar represents a time after the time represented by the specified Object, false otherwise
	 */
	after(obj: Object): boolean

	/**
	 * Indicates if this Calendar represents a time before the time represented by the specified Object.
	 * @param obj - the object to test
	 * @returns true if this Calendar represents a time before the time represented by the specified Object, false otherwise
	 */
	before(obj: Object): boolean

	/**
	 * Sets all the calendar field values and the time value (millisecond offset from the Epoch) of this Calendar undefined.
	 */
	clear(): void

	/**
	 * Sets the given calendar field value and the time value (millisecond offset from the Epoch) of this Calendar undefined.
	 * @param field - the calendar field to be cleared
	 */
	clear(field: Number): void

	/**
	 * Compares the time values (millisecond offsets from the Epoch) represented by two Calendar objects.
	 * @param anotherCalendar - the Calendar to be compared
	 * @returns the value 0 if the time represented by the argument is equal to the time represented by this Calendar; a value less than 0 if the time of this Calendar is before the time represented by the argument; and a value greater than 0 if the time of this Calendar is after the time represented by the argument
	 */
	compareTo(anotherCalendar: Calendar): Number

	/**
	 * Compares two calendar values whether they are equivalent.
	 * @param other - the object to compare against this calendar
	 */
	equals(other: Object): boolean

	/**
	 * Returns the value of the given calendar field.
	 * @param field - the calendar field to retrieve
	 * @returns the value for the given calendar field
	 */
	get(field: Number): Number

	/**
	 * Returns the maximum value that the specified calendar field could have.
	 * @param field - the calendar field
	 * @returns the maximum value that the specified calendar field could have
	 */
	getActualMaximum(field: Number): Number

	/**
	 * Returns the minimum value that the specified calendar field could have.
	 * @param field - the calendar field
	 * @returns the minimum value that the specified calendar field could have
	 */
	getActualMinimum(field: Number): Number

	/**
	 * Returns the first day of the week based on locale context. In the US the first day of the week is SUNDAY. In France the first day of the week is MONDAY.
	 * @returns the first day of the week based on locale context
	 */
	getFirstDayOfWeek(): Number

	/**
	 * Returns the maximum value for the given calendar field.
	 * @param field - the calendar field
	 * @returns the maximum value for the given calendar field
	 */
	getMaximum(field: Number): Number

	/**
	 * Returns the minimum value for the given calendar field.
	 * @param field - the calendar field
	 * @returns the minimum value for the given calendar field
	 */
	getMinimum(field: Number): Number

	/**
	 * Returns the current time stamp of this calendar. This method is also used to convert a Calendar into a Date. WARNING: Keep in mind that the returned Date object's time is always interpreted in the time zone GMT. This means time zone information set at the calendar object will not be honored and gets lost.
	 * @returns the current time stamp of this calendar as a Date
	 */
	getTime(): Date

	/**
	 * Returns the current time zone of this calendar.
	 * @returns the current time zone of this calendar
	 */
	getTimeZone(): String

	/**
	 * Calculates the hash code for a calendar.
	 */
	hashCode(): Number

	/**
	 * Indicates if the specified year is a leap year.
	 * @param year - the year to test
	 * @returns true if the specified year is a leap year
	 */
	isLeapYear(year: Number): boolean

	/**
	 * Checks whether two calendar dates fall on the same day. The method performs comparison based on both calendar's field values by honoring the defined time zones.
	 * @param other - the calendar to compare against this calendar
	 */
	isSameDay(other: Calendar): boolean

	/**
	 * Checks whether two calendar dates fall on the same day. The method performs comparison based on both calendar's time stamps by ignoring any defined time zones.
	 * @param other - the calendar to compare against this calendar
	 */
	isSameDayByTimestamp(other: Calendar): boolean

	/**
	 * Indicates if the field is set.
	 * @param field - the field to test
	 * @returns true if the field is set, false otherwise
	 */
	isSet(field: Number): boolean

	/**
	 * Parses the string according to the date and time format pattern and set the time at this calendar object. If a time zone is included in the format string, this time zone is used to interpret the time. Otherwise the currently set calendar time zone is used to parse the given time string.
	 * @param timeString - the time string to parsed
	 * @param format - the time format string
	 */
	parseByFormat(timeString: String, format: String): void

	/**
	 * Parses the string according the date format pattern of the given locale. If the locale name is invalid, an exception is thrown. The currently set calendar time zone is used to parse the given time string.
	 * @param timeString - the time string to parsed
	 * @param locale - the locale id, which defines the date format pattern
	 * @param pattern - the pattern is one of calendar pattern e.g. SHORT_DATE_PATTERN as defined in the regional settings for the locale
	 */
	parseByLocale(timeString: String, locale: String, pattern: Number): void

	/**
	 * Rolls the specified field up or down one value.
	 * @param field - the field to roll
	 * @param up - if true rolls the field up, if false rolls the field down
	 */
	roll(field: Number, up: boolean): void

	/**
	 * Rolls the specified field using the specified value.
	 * @param field - the field to roll
	 * @param amount - the amount to roll the field
	 */
	roll(field: Number, amount: Number): void

	/**
	 * Sets the given calendar field to the given value.
	 * @param field - the calendar field to set
	 * @param value - the value to set in the field
	 */
	set(field: Number, value: Number): void

	/**
	 * Sets the values for the calendar fields YEAR, MONTH, and DAY_OF_MONTH.
	 * @param year - the value for year
	 * @param month - the value for month
	 * @param date - the value for date
	 */
	set(year: Number, month: Number, date: Number): void

	/**
	 * Sets the values for the calendar fields YEAR, MONTH, DAY_OF_MONTH, HOUR_OF_DAY, and MINUTE.
	 * @param year - the value for year
	 * @param month - the value for month
	 * @param date - the value for date
	 * @param hourOfDay - the value for hour of day
	 * @param minute - the value for minute
	 */
	set(year: Number, month: Number, date: Number, hourOfDay: Number, minute: Number): void

	/**
	 * Sets the values for the calendar fields YEAR, MONTH, DAY_OF_MONTH, HOUR_OF_DAY, MINUTE and SECOND.
	 * @param year - the value for year
	 * @param month - the value for month
	 * @param date - the value for date
	 * @param hourOfDay - the value for hour of day
	 * @param minute - the value for minute
	 * @param second - the value for second
	 */
	set(year: Number, month: Number, date: Number, hourOfDay: Number, minute: Number, second: Number): void

	/**
	 * Sets what the first day of the week is.
	 * @param value - the day to set as the first day of the week
	 */
	setFirstDayOfWeek(value: Number): void

	/**
	 * Sets the current time stamp of this calendar. WARNING: Keep in mind that the set Date object's time is always interpreted in the time zone GMT. This means that time zone information at the calendar object needs to be set separately by using the setTimeZone(String) method.
	 * @param date - the current time stamp of this calendar
	 */
	setTime(date: Date): void

	/**
	 * Sets the current time zone of this calendar. WARNING: Keep in mind that the time stamp represented by the calendar is always interpreted in the time zone GMT. Changing the time zone will not change the calendar's time stamp.
	 * @param timeZone - the current time zone value to set
	 */
	setTimeZone(timeZone: String): void
}
```

```
Object
  dw.util.Calendar
```
