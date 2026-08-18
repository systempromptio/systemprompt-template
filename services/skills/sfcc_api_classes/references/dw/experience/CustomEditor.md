# dw.experience.CustomEditor

## Overview
Represents a Page Designer custom editor for attributes of type `custom`; provides configuration, dependencies and client-side resources.

## Description
Instantiated by Page Designer to supply configuration and resources required by a custom attribute editor UI. Configuration must be JSON-serializable. Dependencies let one custom editor reference others (useful for breakout panels). Resources point to scripts and styles used by the editor UI.

```ts
declare class CustomEditor  {
	/** Configuration values (JSON-serializable). */
	configuration: Map

	/** Mapping of dependent custom editors. */
	dependencies: Map

	/** Resources (scripts/styles) required by the editor UI. */
	resources: CustomEditorResources

	/** Returns the configuration map. */
	getConfiguration(): Map

	/** Returns dependencies mapping (id -> CustomEditor). */
	getDependencies(): Map

	/** Returns resources for the custom editor. */
	getResources(): CustomEditorResources
}
```
