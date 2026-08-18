# dw.system.PipelineDictionary

## Overview
Provides access to pipeline dictionary values via dynamic properties (`pdict.myvalue` or `pdict['myvalue']`).

## Description
Access behavior varies by context: scripts can read/write declared input/output values regardless of aliasing; templates and pipelines access all values. Templates expose the dictionary as `pdict` variable (e.g., `${pdict.Product.ID}`).

Auto-populated values per request include `CurrentSession`, `CurrentRequest`, `CurrentHttpParameterMap`, `CurrentForms`, `CurrentCustomer`, etc.

```ts
declare class PipelineDictionary  {
}
```
