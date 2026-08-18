# dw.template.Velocity

## Overview
Renders Apache Velocity templates (version 1.7) with support for file-based, File object, or inline string templates, optionally writing to custom Writers.

## Description
Renders Apache Velocity templates. Templates can be identified from: a template file name (resolved in Dynamic WebDAV location for current site, must end with '.vm' or '.vs'), a dw.io.File object (any accessible file system location), or a string with inline template content. Files included via `#parse` or `#include` are always resolved in the Dynamic location. By default, output writes to the current response writer, but a dw.io.Writer can be supplied as target. Parameters are passed as a single object with properties. The complete set of VelocityTools are provided for escaping, formatting, and other common tasks. Template files are cached based on instance type.

```ts
declare class Velocity  {
	/**
	 * Includes the rendered content of the specified action URL (pipeline or controller). Must only be used inside a Velocity template.
	 * @param action - the URL (pipeline or controller) to be included
	 * @param namesAndParams - several strings with name=value pairs (e.g., 'pid', 'value1', 'cgid', 'value2')
	 * @returns the string to execute the remote include in the template
	 */
	static remoteInclude(action: string, ...namesAndParams: string[]): string

	/**
	 * Renders an inline template to the response writer.
	 * @param templateContent - the template content
	 * @param args - the argument object
	 */
	static render(templateContent: string, args: Object): void

	/**
	 * Renders an inline template to the provided writer.
	 * @param templateContent - the template content
	 * @param args - the argument object
	 * @param writer - the target writer
	 */
	static render(templateContent: string, args: Object, writer: Writer): void

	/**
	 * Renders a template file to the response writer. Template file name is relative to the current site's Dynamic file location.
	 * @param templateFileName - the file name of the template
	 * @param args - the argument object
	 */
	static renderTemplate(templateFileName: string, args: Object): void

	/**
	 * Renders a template file to the provided writer. Template file name is relative to the current site's Dynamic file location.
	 * @param templateFileName - the file name of the template
	 * @param args - the argument object to pass to the template
	 * @param writer - the target writer
	 */
	static renderTemplate(templateFileName: string, args: Object, writer: Writer): void

	/**
	 * Renders a template file to the response writer.
	 * @param templateFile - the file object denoting the template
	 * @param args - the argument object
	 */
	static renderTemplate(templateFile: File, args: Object): void

	/**
	 * Renders a template file to the provided writer.
	 * @param templateFile - the file object denoting the template
	 * @param args - the argument object
	 * @param writer - the target writer
	 */
	static renderTemplate(templateFile: File, args: Object, writer: Writer): void
}
```
