# dw.experience.ComponentRenderSettings

## Overview
Holds rendering-related settings for a component (templates, caching, markup options, rendering flags).

## Description
Encapsulates configuration controlling how a component is rendered on a page, including caching options, template references, and other presentation flags.

## All Known Subclasses


```ts
declare class ComponentRenderSettings  {
	/** Indicates whether the component output is cacheable. */
	cacheable: boolean

	/** Template or markup reference used to render the component. */
	template: string

	/** Returns whether component output is cacheable. */
	isCacheable(): boolean

	/** Returns the template identifier. */
	getTemplate(): string
}
```
