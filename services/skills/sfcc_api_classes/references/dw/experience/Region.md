# dw.experience.Region

## Overview
Represents a container region that holds components; supports rendering and exposes visible components and size.

## Description
A region contains components and can be rendered via `PageMgr.renderRegion(...)`. Provides the region id, the computed number of components that would render and the collection of visible components. Note that visibility and size are time and customer-group dependent and should not be called from a page-cached context.

```ts
declare class Region  {
	/** Region identifier. */
	ID: string

	/** Number of components that would be rendered (time/customer-group dependent). */
	size: number

	/** Collection of visible components for this region. */
	visibleComponents: Collection

	getID(): string
	getSize(): Number
	getVisibleComponents(): Collection
}
```
