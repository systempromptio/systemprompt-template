# dw.extensions.facebook.FacebookFeedHooks

## Overview
Hooks for customizing Facebook catalog feed exports; executed outside transactions. Define exported JS functions inside a site cartridge and register them in `package.json` (hooks entry points).

## Description
FacebookFeedHooks interface containing extension points for customizing Facebook export feeds. The hooks are registered via a `hooks` JSON and are not executed in a transaction. Each hook maps an extension point name to a script that exports the hook function.

##
```ts
declare class FacebookFeedHooks  {
	/** The extension point name `dw.extensions.facebook.feed.transformProduct`. */
	static extensionPointTransformProduct: 'dw.extensions.facebook.feed.transformProduct'

	/**
	 * Called after default transformation of a Demandware `Product` to `FacebookProduct` for a catalog feed.
	 * Returning a non-null `Status` ends the hook execution.
	 */
	transformProduct(product: Product, facebookProduct: FacebookProduct, feedId: string): Status
}
```
