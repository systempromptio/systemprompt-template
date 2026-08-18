# dw.customer.oauth.OAuthFinalizedResponse

## Overview
Aggregates both the access token response and the user info response returned when finalizing an OAuth login flow.

## Description
Contains the combined responses from the OAuth provider: an `OAuthAccessTokenResponse` and an `OAuthUserInfoResponse`.

```ts
declare class OAuthFinalizedResponse  {
    /** Read-only access token response. */
    accessTokenResponse: dw.customer.oauth.OAuthAccessTokenResponse

    /** Read-only user info response. */
    userInfoResponse: dw.customer.oauth.OAuthUserInfoResponse

    /** Returns the access token response. */
    getAccessTokenResponse(): dw.customer.oauth.OAuthAccessTokenResponse

    /** Returns the user info response. */
    getUserInfoResponse(): dw.customer.oauth.OAuthUserInfoResponse
}
```
