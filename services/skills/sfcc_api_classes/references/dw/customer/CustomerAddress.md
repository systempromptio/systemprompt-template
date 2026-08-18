# dw.customer.CustomerAddress

## Overview
Represents a customer's postal/contact address with getters and setters for typical address fields and helpers.

## Description
Provides access to address fields (first/last name, company, street lines, city, postal code, state, country), contact phone, title/salutation, and helpers to compare addresses. Many setters mutate the persisted address.

```ts
declare class CustomerAddress extends dw.object.PersistentObject {
    /** Returns the first address line. */
    getAddress1(): string

    /** Returns the second address line. */
    getAddress2(): string

    /** Returns the city for the address. */
    getCity(): string

    /** Returns the company name associated with the address. */
    getCompanyName(): string

    /** Returns the country code as an EnumValue (ISO 3166-1 alpha-2). */
    getCountryCode(): EnumValue

    /** Returns the contact's first name. */
    getFirstName(): string

    /** Returns the full name (first, middle, last, suffix concatenated). */
    getFullName(): string

    /** Returns the address identifier (name). */
    getID(): string

    /** Returns the job title for the contact. */
    getJobTitle(): string

    /** Returns the contact's last name. */
    getLastName(): string

    /** Returns the contact phone number. */
    getPhone(): string

    /** Returns the postal / ZIP code. */
    getPostalCode(): string

    /** Returns the post box value if present. */
    getPostBox(): string

    /** Returns the salutation (e.g., Mr., Ms.). */
    getSalutation(): string

    /** Returns the contact's middle/second name. */
    getSecondName(): string

    /** Returns the state/province code. */
    getStateCode(): string

    /** Returns the name suffix (e.g., Jr., Sr.). */
    getSuffix(): string

    /** Returns the suite/apartment value. */
    getSuite(): string

    /** Returns the title associated with the address. */
    getTitle(): string

    /**
     * Returns true when the provided object represents an address whose
     * core attributes (address1, address2, city, companyName, countryCode,
     * firstName, lastName, postalCode, postBox, stateCode) are equal.
     * @param address - object to compare
     */
    isEquivalentAddress(address: Object): boolean

    /** Sets the first address line. */
    setAddress1(value: string): void

    /** Sets the second address line. */
    setAddress2(value: string): void

    /** Sets the city. */
    setCity(city: string): void

    /** Sets the company name. */
    setCompanyName(companyName: string): void

    /** Sets the country code (ISO 3166-1 alpha-2). */
    setCountryCode(countryCode: string): void

    /** Sets the contact's first name. */
    setFirstName(firstName: string): void

    /** Sets the address ID/name. */
    setID(value: string): void

    /** Sets the job title. */
    setJobTitle(jobTitle: string): void

    /** Sets the contact's last name. */
    setLastName(lastName: string): void

    /** Sets the contact phone number (max length enforced by platform). */
    setPhone(phoneNumber: string): void

    /** Sets the postal / ZIP code. */
    setPostalCode(postalCode: string): void

    /** Sets the post box value. */
    setPostBox(postBox: string): void

    /** Deprecated misspelling; sets the salutation. Use `setSalutation`. */
    setSalutation(value: string): void

    /** Sets the contact's middle/second name. */
    setSecondName(secondName: string): void

    /** Sets the state/province code. */
    setStateCode(state: string): void

    /** Sets the name suffix (e.g., Jr., Sr.). */
    setSuffix(suffix: string): void

    /** Sets the suite/apartment value (max length enforced). */
    setSuite(value: string): void

    /** Sets the title associated with the address. */
    setTitle(title: string): void
}
```
