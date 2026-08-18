# dw.extensions.pinterest.PinterestOrderHooks

## Overview
Hook interface for Pinterest order processing extension points.

## Description
Defines extension points related to Pinterest order processing. Implementations should be provided in site cartridges as exported functions and registered via hooks.json.

```ts
declare class PinterestOrderHooks {
  /** Called to transform or validate Pinterest orders; implementations return a Status to control execution. */
  someHookMethod(...args: any[]): dw.system.Status
}
```

All Known Subclasses

None
