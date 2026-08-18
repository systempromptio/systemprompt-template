# dw.experience.AspectAttributeValidationException

## Overview
Exception indicating validation errors for aspect attributes in Experience/Component configurations.

## Description
Thrown when an aspect attribute fails validation (for example during component configuration or rendering). Contains information about the validation failure.

## All Known Subclasses


```ts
declare class AspectAttributeValidationException extends Error {
	/** Message describing the validation failure. */
	message: string
}
```
