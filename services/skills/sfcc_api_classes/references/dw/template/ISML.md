# dw.template.ISML

## Overview
Renders ISML templates to the current response, supporting template arguments via pdict and controlling response behavior through ISML tags.

## Description
Provides support for rendering ISML templates. Templates are stored as *.isml files in locale-specific folders under '/cartridge/templates', with '/cartridge/template/default' being the default locale. Template name arguments represent the template path (without file ending) within this folder structure. Templates may contain ISML tags which control character encoding, content type, caching behavior, and other response settings.

```ts
declare class ISML  {
	/**
	 * Renders an ISML template and writes the output to the current response.
	 * @deprecated No longer available as of version 17.7. Puts template arguments into the global pipeline dictionary.
	 * @param template - the template path
	 */
	static renderTemplate(template: string): void

	/**
	 * Renders an ISML template and writes the output to the current response. Template arguments are accessible via pdict.* in the template.
	 * @deprecated No longer available as of version 17.7. Puts template arguments into the global pipeline dictionary.
	 * @param template - the template path
	 * @param templateArgs - the template arguments object
	 */
	static renderTemplate(template: string, templateArgs: Object): void

	/**
	 * Renders an ISML template and writes the output to the current response. Keeps template arguments in a local pipeline dictionary scope (available from version 17.7).
	 * @param template - the template path
	 */
	static renderTemplate(template: string): void

	/**
	 * Renders an ISML template and writes the output to the current response. Template arguments are accessible via pdict.* in the template. Keeps template arguments in a local pipeline dictionary scope (available from version 17.7).
	 * @param template - the template path
	 * @param templateArgs - the template arguments object
	 */
	static renderTemplate(template: string, templateArgs: Object): void
}
```
