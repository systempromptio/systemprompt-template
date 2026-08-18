# dw.experience.RegionRenderSettings

## Overview
Configures how a Region is rendered: wrapper tag name and wrapper attributes, plus default and per-component render settings.

## Description
A configuration object that controls rendering of a Region. You can set the wrapper element tag name (defaults to `div`), provide attributes for the wrapper element (for example `class`), and specify default or per-component `ComponentRenderSettings` used when rendering components contained in the region.

## All Known Subclasses


```ts
declare class RegionRenderSettings  {
	/**
	 * The configured attributes of the wrapper element as set by `setAttributes`.
	 */
	attributes: Object

	/**
	 * The default component render settings applied to components in this region.
	 */
	defaultComponentRenderSettings: ComponentRenderSettings

	/**
	 * The tag name used for the region wrapper element (defaults to 'div').
	 */
	tagName: string

	/**
	 * Creates region render settings which can then be configured further.
	 */
	constructor(): RegionRenderSettings

	/**
	 * Returns the configured attributes of the wrapper element.
	 */
	getAttributes(): Object

	/**
	 * Returns the component render settings for the given component (or default if none set).
	 * @param component - component to retrieve settings for
	 */
	getComponentRenderSettings(component: Component): ComponentRenderSettings

	/**
	 * Returns the default component render settings.
	 */
	getDefaultComponentRenderSettings(): ComponentRenderSettings

	/**
	 * Returns the tag name of the region wrapper element.
	 */
	getTagName(): string

	/**
	 * Sets the attributes of the wrapper element (map of String->String). Use `null` to use system defaults.
	 * @param attributes - attributes map
	 */
	setAttributes(attributes: Object): RegionRenderSettings

	/**
	 * Sets the component render settings for the given component.
	 */
	setComponentRenderSettings(component: Component, componentRenderSettings: ComponentRenderSettings): RegionRenderSettings

	/**
	 * Sets the default component render settings used for components in this region.
	 */
	setDefaultComponentRenderSettings(defaultComponentRenderSettings: ComponentRenderSettings): RegionRenderSettings

	/**
	 * Sets the tag name of the region wrapper element. Must not be empty.
	 */
	setTagName(tagName: string): RegionRenderSettings
}
```
