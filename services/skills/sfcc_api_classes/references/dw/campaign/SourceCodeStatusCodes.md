# dw.campaign.SourceCodeStatusCodes

## Overview
Contains string constants used as result/error codes by the SetSourceCode pipelet.

## Description
Provides named string codes that indicate whether a source code was invalid or found but inactive.

```ts
declare class SourceCodeStatusCodes  {
    /** Indicates the specified source code was found but no matching active groups exist. */
    static CODE_INACTIVE: 'CODE_INACTIVE'

    /** Indicates the specified source code is not contained in any group. */
    static CODE_INVALID: 'CODE_INVALID'
}
```

## All Known Subclasses
None
