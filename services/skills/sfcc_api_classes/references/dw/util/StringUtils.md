# dw.util.StringUtils

## Overview
Provides utility methods for string manipulation, formatting, encoding, and conversion operations.

## Description
String utility class.

```
Object
  dw.util.StringUtils
```

```ts
declare class StringUtils  {
	/**
	 * String encoding type HTML.
	 */
	static ENCODE_TYPE_HTML: 0

	/**
	 * String encoding type XML.
	 */
	static ENCODE_TYPE_XML: 1

	/**
	 * String encoding type WML.
	 * @deprecated Don't use this constant anymore
	 */
	static ENCODE_TYPE_WML: 2

	/**
	 * String truncate mode 'char'. Truncate string to the nearest character. Default mode if no truncate mode is specified.
	 */
	static TRUNCATE_CHAR: 'char'

	/**
	 * String truncate mode 'sentence'. Truncate string to the nearest sentence.
	 */
	static TRUNCATE_SENTENCE: 'sentence'

	/**
	 * String truncate mode 'word'. Truncate string to the nearest word.
	 */
	static TRUNCATE_WORD: 'word'

	/**
	 * Interprets a Base64 encoded string as byte stream of an UTF-8 encoded string.
	 * @param base64 - the Base64 encoded string
	 * @returns the decoded string
	 */
	static decodeBase64(base64: string): string

	/**
	 * Interprets a Base64 encoded string as the byte stream representation of a string with specified character encoding.
	 * @param base64 - the Base64 encoded string
	 * @param characterEncoding - the character encoding to read the input string
	 * @returns the decoded string
	 */
	static decodeBase64(base64: string, characterEncoding: string): string

	/**
	 * Convert a given syntax-safe string to a string according to the selected character entity encoding type.
	 * @param str - String to be decoded
	 * @param type - decode type
	 * @returns decoded string
	 */
	static decodeString(str: string, type: Number): string

	/**
	 * Encodes the byte representation of the given string as Base64 using UTF-8 encoding.
	 * @param str - the string to encode
	 * @returns the encoded string
	 */
	static encodeBase64(str: string): string

	/**
	 * Encodes the byte representation of the given string as Base64 using specified character encoding.
	 * @param str - the string to encode
	 * @param characterEncoding - the character encoding to read the input string
	 * @returns the encoded string
	 */
	static encodeBase64(str: string, characterEncoding: string): string

	/**
	 * Convert a given string to a syntax-safe string according to the selected character entity encoding type.
	 * @param str - String to be encoded
	 * @param type - encode type
	 * @returns encoded string
	 */
	static encodeString(str: string, type: Number): string

	/**
	 * Returns a formatted string using the specified format and arguments (Java MessageFormat).
	 * @param format - Java like formatting string
	 * @param args - optional list of arguments or a collection, which are included into the result string
	 * @returns the formatted result string
	 */
	static format(format: string, ...args: Object): string

	/**
	 * Formats a Calendar object with Calendar.INPUT_DATE_TIME_PATTERN format of the current request locale.
	 * @param calendar - the calendar object
	 * @returns a string representation of the formatted calendar object
	 */
	static formatCalendar(calendar: Calendar): string

	/**
	 * Formats a Calendar object with the provided date format.
	 * @param calendar - the calendar object to be printed
	 * @param format - the format to use
	 * @returns a string representation of the formatted calendar object
	 */
	static formatCalendar(calendar: Calendar, format: string): string

	/**
	 * Formats a Calendar object with the date format defined by the provided locale and Calendar pattern.
	 * @param calendar - the calendar object to be printed
	 * @param locale - the locale, which defines the date format to be used
	 * @param pattern - the pattern is one of a calendar pattern e.g. SHORT_DATE_PATTERN
	 * @returns a string representation of the formatted calendar object
	 */
	static formatCalendar(calendar: Calendar, locale: string, pattern: Number): string

	/**
	 * Formats a date with the default date format of the current site.
	 * @deprecated Use formatCalendar(Calendar, String) instead
	 * @param date - the date to format
	 * @returns a string representation of the formatted date
	 */
	static formatDate(date: Date): string

	/**
	 * Formats a date with the provided date format.
	 * @deprecated Use formatCalendar(Calendar, String) instead
	 * @param date - the date to format
	 * @param format - the format to use
	 * @returns a string representation of the formatted date
	 */
	static formatDate(date: Date, format: string): string

	/**
	 * Formats a date with the provided date format in specified locale.
	 * @deprecated Use formatCalendar(Calendar, String) instead
	 * @param date - the date to format
	 * @param format - the format to use
	 * @param locale - the locale to use
	 * @returns a string representation of the formatted date
	 */
	static formatDate(date: Date, format: string, locale: string): string

	/**
	 * Returns a formatted integer number using the default integer format of the current site.
	 * @param number - the number to format
	 * @returns a formatted an integer number with the default integer format of the current site
	 */
	static formatInteger(number: Number): string

	/**
	 * Formats a Money Object with the default money format of the current request locale.
	 * @param money - The Money instance that should be formatted
	 * @returns The formatted String representation of the passed money
	 */
	static formatMoney(money: Money): string

	/**
	 * Returns a formatted number using the default number format of the current site.
	 * @param number - the number to format
	 * @returns a formatted number using the default number format of the current site
	 */
	static formatNumber(number: Number): string

	/**
	 * Returns a formatted string using the specified number and format.
	 * @param number - the number to format
	 * @param format - the format to use
	 * @returns a formatted string using the specified number and format
	 */
	static formatNumber(number: Number, format: string): string

	/**
	 * Returns a formatted number as a string using the specified number format in specified locale.
	 * @param number - the number to format
	 * @param format - the format to use
	 * @param locale - the locale to use
	 * @returns a formatted number as a string using the specified number format in specified locale
	 */
	static formatNumber(number: Number, format: string, locale: string): string

	/**
	 * Return a string in which specified number of characters in the suffix is not changed and the rest of the characters replaced with specified character.
	 * @param str - String to garble
	 * @param replaceChar - character to use as a replacement
	 * @param suffixLength - length of the suffix
	 * @returns the garbled string
	 */
	static garble(str: string, replaceChar: string, suffixLength: Number): string

	/**
	 * Returns the string with leading white space removed.
	 * @param str - the String to remove characters from
	 * @returns the string with leading white space removed
	 */
	static ltrim(str: string): string

	/**
	 * This method provides cell padding functionality to the template.
	 * @param str - the string to process
	 * @param width - The absolute value defines the width of the cell. Positive number forces left, negative right alignment
	 * @returns the processed string
	 */
	static pad(str: string, width: Number): string

	/**
	 * Returns the string with trailing white space removed.
	 * @param str - the String to remove characters from
	 * @returns the string with trailing white space removed
	 */
	static rtrim(str: string): string

	/**
	 * Convert a given string to an HTML-safe string.
	 * @param str - String to be converted
	 * @returns converted string
	 */
	static stringToHtml(str: string): string

	/**
	 * Converts a given string to a WML-safe string.
	 * @deprecated Don't use this method anymore
	 * @param str - String to be converted
	 * @returns the converted string
	 */
	static stringToWml(str: string): string

	/**
	 * Converts a given string to a XML-safe string.
	 * @param str - String to be converted
	 * @returns the converted string
	 */
	static stringToXml(str: string): string

	/**
	 * Returns the string with leading and trailing white space removed.
	 * @param str - the string to trim
	 * @returns the string with leading and trailing white space removed
	 */
	static trim(str: string): string

	/**
	 * Truncate the string to the specified length using specified truncate mode. Optionally append suffix to truncated string.
	 * @param str - string to truncate
	 * @param maxLength - maximum length of the truncated string, not including suffix
	 * @param mode - truncate mode (TRUNCATE_CHAR, TRUNCATE_WORD, TRUNCATE_SENTENCE), if null TRUNCATE_CHAR is assumed
	 * @param suffix - suffix append to the truncated string
	 * @returns the truncated string
	 */
	static truncate(str: string, maxLength: Number, mode: string, suffix: string): string
}
```
