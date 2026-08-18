# dw.experience.ComponentScriptContext

## Overview
Context object passed to component type `render` and `serialize` functions; exposes the current component, its render settings and processed content attributes.

## Description
This context is handed to the `render` and `serialize` functions of a component type script. It provides access to the component being rendered, the effective component render settings, and the processed content attributes (expansion + conversion) usable by the script.

```ts
declare class ComponentScriptContext  {
	/** The component for which this script is executed. */
	component: Component

	/** The render settings for the current component. */
	componentRenderSettings: ComponentRenderSettings

	/** Processed content attributes (expanded and converted). */
	content: Map

	/**
	 * Returns the component for which the corresponding component type script is currently executed.
	 */
	getComponent(): Component

	/**
	 * Returns the component render settings available to the component script.
	 */
	getComponentRenderSettings(): ComponentRenderSettings

	/**
	 * Returns processed content attributes of the component (expansion + conversion applied).
	 */
	getContent(): Map
}
```
