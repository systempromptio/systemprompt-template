# dw.util.Template

## Overview
Reads ISML templates from the file system and renders them into MimeEncodedText objects with optional substitution parameters.

## Description
Reads an ISML template from the file system and renders it into a MimeEncodedText object. Optional substitution values can be passed to the ISML template via the render(Map) method. Substitution parameters can be accessed within the template through `${param.parameter}` or for backward compatibility through `${pdict.parameter}`. The pdict access only gives access to the parameter map provided at rendering time and doesn't offer access to the system PipelineDictionary.

```
Object
  dw.util.Template
```

```ts
declare class Template  {
	/**
	 * Creates a new template. Doesn't render until render() or render(Map) is invoked.
	 * @param templateName - file system path to the ISML template
	 */
	constructor(templateName: string)

	/**
	 * Creates a new template with the locale set to the given localeID.
	 * @param templateName - file system path to the ISML template
	 * @param localeID - localeID to be used for Rendering
	 */
	constructor(templateName: string, localeID: string)

	/**
	 * Renders the template specified at instantiation time, without any substitution parameters.
	 * @returns MimeEncodedText with isprint tags referring to param/pdict replaced with empty String
	 */
	render(): MimeEncodedText

	/**
	 * Renders the template specified at instantiation time with the given substitution parameters.
	 * @param params - Map of substitution parameters accessible through param or pdict variables in ISML
	 * @returns MimeEncodedText containing the rendered template
	 */
	render(params: Map): MimeEncodedText

	/**
	 * Sets an optional localeID which is used instead of the current request's localeID.
	 * @param localeID - to be used for processing this template
	 * @returns this Template object
	 */
	setLocale(localeID: string): Template
}
```
