# dw.experience.image.FocalPoint

## Overview
Represents an image focal point with X and Y coordinates.

## Description
A small value object exposing the focal point coordinates (abscissa `x` and ordinate `y`) for an image. Instances are read-only and typically returned by the `Image` API.

## All Known Subclasses


```ts
declare class FocalPoint  {
	/**
	 * Focal point abscissa (read-only).
	 */
	x: number

	/**
	 * Focal point ordinate (read-only).
	 */
	y: number

	/**
	 * Returns the focal point abscissa.
	 */
	getX(): number

	/**
	 * Returns the focal point ordinate.
	 */
	getY(): number
}
```
