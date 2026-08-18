# dw.svc.HTTPFormService

## Overview
HTTP service for sending URL-encoded form data via POST requests.

## Description
All arguments passed to the call method will be URL-encoded and set as name/value pairs in the HTTP request body. The HTTP request will be a POST with a content-type of `application/x-www-form-urlencoded`.

```
Object
  dw.svc.Service
    dw.svc.HTTPService
      dw.svc.HTTPFormService
```

```ts
declare class HTTPFormService extends HTTPService {
}
```
