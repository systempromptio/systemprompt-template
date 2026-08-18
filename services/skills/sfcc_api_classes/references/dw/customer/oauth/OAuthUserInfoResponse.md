# dw.customer.oauth.OAuthUserInfoResponse

## Overview
Represents the user information response returned by an OAuth provider when requesting user profile data.

## Description
The content and format of `userInfo` vary by provider and configured scope; typically it's a JSON string. Provides access to raw user info and error status.

```ts
declare class OAuthUserInfoResponse  {
    /** Read-only error status or null. */
    errorStatus: string

    /** Read-only user info string (commonly JSON). */
    userInfo: string

    /** Returns the error status or null. */
    getErrorStatus(): string

    /** Returns the user info string (format depends on provider and scope). */
    getUserInfo(): string
}
```
