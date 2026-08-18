# dw.system.StatusItem

## Overview
Holds the data for a single status entry, then Status bundles multiple StatusItems into an overall result.

## Description
Each StatusItem records a code, message, parameters, and optional details; the message is only a default human readable string and should not be used to identify the status, so callers rely on getCode() or Status.OK / Status.ERROR instead.

```ts
declare class StatusItem  {
    /**
     * The unique status code that can be used to look up a localized message.
     */
    code: string

    /**
     * @readonly
     * Optional detail entries for this StatusItem.
     */
    readonly details: Map

    /**
     * @readonly
     * True when this StatusItem represents an ERROR.
     */
    readonly error: boolean

    /**
     * The default human readable message, which may change between releases.
     */
    message: string

    /**
     * The parameters used to construct a custom message.
     */
    parameters: List

    /**
     * Either Status.OK or Status.ERROR.
     */
    status: number

    /**
     * Adds a detail entry to this StatusItem.
     */
    addDetail(key: string, value: Object): void

    /**
     * Returns the status code.
     */
    getCode(): string

    /**
     * Returns the optional details map for this StatusItem.
     */
    getDetails(): Map

    /**
     * Returns the default human readable message.
     */
    getMessage(): string

    /**
     * Returns the parameters for this StatusItem.
     */
    getParameters(): List

    /**
     * Returns either Status.OK or Status.ERROR.
     */
    getStatus(): number

    /**
     * Returns true when this item represents an error.
     */
    isError(): boolean

    /**
     * Updates the status code for this item.
     */
    setCode(code: string): void

    /**
     * Updates the default human readable message for this item.
     */
    setMessage(message: string): void

    /**
     * Replaces the parameters for this item with the provided values.
     */
    setParameters(...parameters: Object[]): void

    /**
     * Updates the numeric status, typically Status.OK or Status.ERROR.
     */
    setStatus(status: number): void
}
```