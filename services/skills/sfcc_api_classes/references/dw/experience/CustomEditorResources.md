# dw.experience.CustomEditorResources

## Overview
Holds lists of script and style resource URLs required by a `CustomEditor` (client-side assets for Page Designer UI).

## Description
Represents script and style resources for a custom editor. Relative paths are resolved to absolute static URLs; lists may be adjusted at runtime in the custom editor `init` function.

```ts
declare class CustomEditorResources  {
	/** Script resource URLs required by the custom editor. */
	scripts: List

	/** Style resource URLs required by the custom editor. */
	styles: List

	/** Returns script URLs (never null). */
	getScripts(): List

	/** Returns style URLs (never null). */
	getStyles(): List
}
```
