# dw.extensions.facebook.FacebookProduct

## Overview
Represents a row in the Facebook catalog feed export; container for product metadata used by Facebook.

## Description
A data object representing a product row in the Facebook catalog feed. Includes predefined constant values for age groups, availability, conditions, shipping units, and read/write properties for typical feed fields (title, description, price, images, etc.).

##
```ts
declare class FacebookProduct  {
	/** Age group constant */
	static AGE_GROUP_ADULT: 'adult'
	static AGE_GROUP_INFANT: 'infant'
	static AGE_GROUP_KIDS: 'kids'
	static AGE_GROUP_NEWBORN: 'newborn'
	static AGE_GROUP_TODDLER: 'toddler'

	/** Availability constants */
	static AVAILABILITY_AVAILABLE_FOR_ORDER: 'available for order'
	static AVAILABILITY_IN_STOCK: 'in stock'
	static AVAILABILITY_OUT_OF_STOCK: 'out of stock'
	static AVAILABILITY_PREORDER: 'preorder'

	/** Condition constants */
	static CONDITION_NEW: 'new'
	static CONDITION_REFURBISHED: 'refurbished'
	static CONDITION_USED: 'used'

	/** Gender constants */
	static GENDER_FEMALE: 'female'
	static GENDER_MALE: 'male'
	static GENDER_UNISEX: 'unisex'

	/** Shipping size/weight units */
	static SHIPPING_SIZE_UNIT_CM: 'cm'
	static SHIPPING_SIZE_UNIT_FT: 'ft'
	static SHIPPING_SIZE_UNIT_IN: 'in'
	static SHIPPING_SIZE_UNIT_M: 'm'
	static SHIPPING_WEIGHT_UNIT_G: 'g'
	static SHIPPING_WEIGHT_UNIT_KG: 'kg'
	static SHIPPING_WEIGHT_UNIT_LB: 'lb'
	static SHIPPING_WEIGHT_UNIT_OZ: 'oz'

	/** Properties (examples, not exhaustive) */
	ageGroup: string
	availability: string
	brand: string
	color: string
	condition: string
	customLabel0: string
	customLabel1: string
	customLabel2: string
	customLabel3: string
	customLabel4: string
	description: string
	expirationDate: Date
	gender: string
	googleProductCategory: string
	gtin: string
	ID: string
	imageLinks: List
	itemGroupID: string
	link: URL
	material: string
	mpn: string
	pattern: string
	price: Money
	productType: string
	salePrice: Money
	salePriceEffectiveDateEnd: Date
	salePriceEffectiveDateStart: Date
	shippingHeight: number
	shippingLength: number
	shippingSizeUnit: string
	shippingWeight: Quantity
	shippingWidth: number
	size: string
	title: string

	/** Getters */
	getAgeGroup(): string
	getAvailability(): string
	getBrand(): string
	getColor(): string
	getCondition(): string
	getDescription(): string
	getGTIN(): string
	getID(): string
	getImageLinks(): List
	getLink(): URL
	getPrice(): Money
	getSalePrice(): Money
	getTitle(): string
}
```
