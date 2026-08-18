# dw.experience.PageScriptContext

## Overview
Context passed to page type `render` and `serialize` functions; exposes the page, processed content attributes and runtime parameters.

## Description
This context is passed to page type scripts during rendering and serialization. It provides the processed content attributes (expansion + conversion), the page being executed, and runtime/render parameters.

```ts
declare class PageScriptContext  {
	/** Processed content attributes (expanded + converted). */
	content: Map

	/** The page for which the script is executed. */
	page: Page

	/** Deprecated: render parameters as string. */
	renderParameters: string

	/** Runtime parameters passed to rendering/serialization. */
	runtimeParameters: string

	getContent(): Map
	getPage(): Page
	getRenderParameters(): string
	getRuntimeParameters(): string
}
```
