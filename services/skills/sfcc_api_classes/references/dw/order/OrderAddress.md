# dw.order.OrderAddress

## Overview
Represents a customer's postal/contact address used on orders.

## Description
The Address class represents a customer's address. Provides getters and setters
for name, street, city, postal code, country and related fields.

## All Known Subclasses

```ts
declare class OrderAddress extends ExtensibleObject {
    /** The customer's first address. */
    address1: string

    /** The customer's second address. */
    address2: string

    /** The Customer's City. */
    city: string

    /** The Customer's company name. */
    companyName: string

    /** The customer's country code. */
    countryCode: EnumValue

    /** The Customer's first name. */
    firstName: string

    /** A concatenation of the Customer's first, middle, and last names and its suffix. (read-only) */
    /** @readonly */
    fullName: string

    /** The customer's job title. */
    jobTitle: string

    /** The customer's last name. */
    lastName: string

    /** The customer's phone number. */
    phone: string

    /** The customer's postal code. */
    postalCode: string

    /** The customer's post box. */
    postBox: string

    /** The customer's salutation. */
    salutation: string

    /** The customer's second name. */
    secondName: string

    /** The customer's state. */
    stateCode: string

    /** The customer's suffix. */
    suffix: string

    /** The customer's suite. */
    suite: string

    /** The customer's title. */
    title: string

    /** Returns the customer's first address. */
    getAddress1(): string

    /** Returns the customer's second address. */
    getAddress2(): string

    /** Returns the Customer's City. */
    getCity(): string

    /** Returns the Customer's company name. */
    getCompanyName(): string

    /** Returns the customer's country code. */
    getCountryCode(): EnumValue

    /** Returns the Customer's first name. */
    getFirstName(): string

    /** Returns a concatenation of the Customer's names. */
    getFullName(): string

    /** Returns the customer's job title. */
    getJobTitle(): string

    /** Returns the customer's last name. */
    getLastName(): string

    /** Returns the customer's phone number. */
    getPhone(): string

    /** Returns the customer's postal code. */
    getPostalCode(): string

    /** Returns the customer's post box. */
    getPostBox(): string

    /** Returns the customer's salutation. */
    getSalutation(): string

    /** Returns the customer's second name. */
    getSecondName(): string

    /** Returns the customer's state. */
    getStateCode(): string

    /** Returns the customer's suffix. */
    getSuffix(): string

    /** Returns the customer's suite. */
    getSuite(): string

    /** Returns the customer's title. */
    getTitle(): string

    /** Returns true if the specified address is equivalent to this address. */
    isEquivalentAddress(address: unknown): boolean

    /** Sets the customer's first address. */
    setAddress1(value: string): void

    /** Sets the customer's second address. */
    setAddress2(value: string): void

    /** Sets the Customer's City. */
    setCity(city: string): void

    /** Sets the Customer's company name. */
    setCompanyName(companyName: string): void

    /** Sets the Customer's country code. */
    setCountryCode(countryCode: string): void

    /** Sets the Customer's first name. */
    setFirstName(firstName: string): void

    /** Sets the customer's job title. */
    setJobTitle(jobTitle: string): void

    /** Sets the customer's last name. */
    setLastName(lastName: string): void

    /** Sets the customer's phone number. */
    setPhone(phoneNumber: string): void

    /** Sets the customer's postal code. */
    setPostalCode(postalCode: string): void

    /** Sets the customer's post box. */
    setPostBox(postBox: string): void

    /** Sets the customer's salutation. */
    setSalutation(value: string): void

    /** Sets the customer's second name. */
    setSecondName(secondName: string): void

    /** Sets the customer's state. */
    setStateCode(state: string): void

    /** Sets the customer's suffix. */
    setSuffix(suffix: string): void

    /** Sets the customer's suite. */
    setSuite(value: string): void

    /** Sets the customer's title. */
    setTitle(title: string): void
}
```
