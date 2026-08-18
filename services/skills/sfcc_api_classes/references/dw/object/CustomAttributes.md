# dw.object.CustomAttributes

## Overview
Read/write container for custom attributes exposed as ECMA properties.

## Description
Used with objects that expose custom attributes. Attributes are accessible via `custom` as normal ECMA properties.
Single-valued attributes are assigned directly; multi-valued attributes use arrays. Multi-value arrays returned are read-only.

```ts
declare class CustomAttributes  {
    /** Access attributes via property names. Example: `eo.custom.svalue = "str"`. */
    [name: string]: any
}
```
