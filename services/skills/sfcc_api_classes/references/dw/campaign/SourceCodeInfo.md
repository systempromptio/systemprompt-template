# dw.campaign.SourceCodeInfo

## Overview
Represents a single applied source code (a literal code) and its status, group, and optional redirect information.

## Description
Contains the literal source-code string, a reference to its SourceCodeGroup, a URL redirect (if configured), and a numeric status flag indicating validity/active state.

```ts
declare class SourceCodeInfo  {
    /** STATUS_INVALID: the source-code is not found in the system. */
    static STATUS_INVALID: 0

    /** STATUS_INACTIVE: the source-code is found but not active. */
    static STATUS_INACTIVE: 1

    /** STATUS_ACTIVE: the source-code is found and active. */
    static STATUS_ACTIVE: 2

    /** The literal source-code. */
    readonly code: string

    /** Associated SourceCodeGroup. */
    readonly group: SourceCodeGroup

    /** Redirect information for the source code (may be null). */
    readonly redirect: URLRedirect

    /** Numeric status (one of STATUS_* constants). */
    readonly status: number

    /** Returns the source-code string. */
    getCode(): string

    /** Returns the associated SourceCodeGroup. */
    getGroup(): SourceCodeGroup

    /** Returns redirect information resolved for this source code, or null. */
    getRedirect(): URLRedirect

    /** Returns numeric status for the source code. */
    getStatus(): number
}
```

## All Known Subclasses
None
