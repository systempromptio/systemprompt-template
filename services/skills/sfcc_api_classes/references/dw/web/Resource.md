# dw.web.Resource

## Overview
Provides methods for retrieving locale-specific messages from properties resource bundles.

## Description
Library class for accessing messages from properties resource bundles containing locale-specific strings. Resources are associated with cartridge templates in the `template/resources` directory. Supports standard Java ResourceBundle lookup rules based on the current request locale. Properties files use UTF-8 encoding and support Unicode escape sequences.

```ts
declare class Resource {
	/**
	 * Returns the message from the default properties resource bundle (base name "message") corresponding to the specified key and the request locale.
	 * @param key - Resource bundle message key
	 * @returns The resource bundle message or the key itself if no message is defined
	 */
	static msg(key: string): string

	/**
	 * Returns the message from the default properties resource bundle (base name "message") corresponding to the specified key and the request locale. If no message for the key is found, returns the default message if it is not null, otherwise returns the key itself.
	 * @param key - Resource bundle message key
	 * @param defaultMessage - Default message to return if no message corresponding to the key is found
	 * @returns The resource bundle message or default message
	 */
	static msg(key: string, defaultMessage: string): string

	/**
	 * Returns the message from the specified properties resource bundle. The resource bundle is located by iterating the site cartridges and looking for a bundle with the specified name in the cartridge template/resources directory.
	 * @param key - Resource bundle message key
	 * @param bundleName - Base bundle name, if null, default bundle name "message" is used
	 * @param defaultMessage - Default message to return if no message corresponding to the key is found and defaultMessage is not null
	 * @returns The resource bundle message or default message
	 * @throws Exception if the key is null
	 */
	static msg(key: string, bundleName: string, defaultMessage: string): string

	/**
	 * Returns the message from the specified properties resource bundle, with the provided arguments substituted for the message argument placeholders (specified using the Java MessageFormat approach).
	 * @param key - Resource bundle message key
	 * @param bundleName - Base bundle name, if null, default bundle name "message" is used
	 * @param defaultMessage - Default message to return if no message corresponding to the key is found and defaultMessage is not null
	 * @param args - Optional list of arguments or a collection, which are included into the result string
	 * @returns The resource bundle message or default message
	 */
	static msgf(key: string, bundleName: string, defaultMessage: string, ...args: Object[]): string
}
```
