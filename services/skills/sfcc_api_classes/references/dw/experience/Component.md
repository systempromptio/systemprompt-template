# dw.experience.Component

## Overview
Represents a page component (experience fragment) used by the CMS/Experience Framework to render content and manage component settings.

## Description
Encapsulates component metadata and behavior for rendering within the storefront experience system. Components include render settings, script contexts, and editors.

## All Known Subclasses


```ts
declare class Component  {
	/** Component identifier. */
	id: string

	/** Component type or template name. */
	type: string

	/** Returns render settings for the component. */
	getRenderSettings(): ComponentRenderSettings

	/** Returns script/context used during server-side rendering. */
	getScriptContext(): ComponentScriptContext

	/** Returns editor metadata if available. */
	getEditorResources(): CustomEditorResources
}
```
