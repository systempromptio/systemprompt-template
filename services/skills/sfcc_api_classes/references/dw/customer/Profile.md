# dw.customer.Profile

## Overview
Represents a customer's profile data (names, contact numbers, locale, birthday, tax ID, etc.) and provides getters and setters for profile attributes.

## Description
Customer profile holds personal and contact details used across storefront and account features. Many fields are readable and writable with appropriate context; some methods return masked values when security restrictions apply.

```ts
declare class Profile  {
    /** Returns the customer's first name. */
    getFirstName(): string

    /** Returns the customer's gender as an EnumValue. */
    getGender(): dw.value.EnumValue

    /** Returns the customer's job title. */
    getJobTitle(): string

    /** Returns the customer's last login time. */
    getLastLoginTime(): Date

    /** Returns the customer's last name. */
    getLastName(): string

    /** Returns the customer's last visit time (remember-me enabled). */
    getLastVisitTime(): Date

    /** Returns the customer's next birthday as a Date or null. */
    getNextBirthday(): Date

    /** Returns the customer's business phone number. */
    getPhoneBusiness(): string

    /** Returns the customer's home phone number. */
    getPhoneHome(): string

    /** Returns the customer's mobile phone number. */
    getPhoneMobile(): string

    /** Returns the customer's preferred locale. */
    getPreferredLocale(): string

    /** Returns the customer's previous login time. */
    getPreviousLoginTime(): Date

    /** Returns the customer's previous visit time. */
    getPreviousVisitTime(): Date

    /** Returns the customer's salutation. */
    getSalutation(): string

    /** Returns the customer's second name. */
    getSecondName(): string

    /** Returns the customer's suffix (e.g., "Jr."). */
    getSuffix(): string

    /** Returns the tax ID; may be masked depending on context. */
    getTaxID(): string

    /** Returns the masked tax ID. */
    getTaxIDMasked(): string

    /** Returns the tax ID type. */
    getTaxIDType(): dw.value.EnumValue

    /** Returns the customer's title (e.g., "Mr"). */
    getTitle(): string

    /** Returns the wallet associated with this profile. */
    getWallet(): dw.customer.Wallet

    /** True if customer is female. */
    isFemale(): boolean

    /** True if customer is male. */
    isMale(): boolean

    /** Sets the customer's birthday. */
    setBirthday(aValue: Date): void

    /** Sets the customer's company name. */
    setCompanyName(aValue: string): void

    /** Sets the customer's email address. */
    setEmail(aValue: string): void

    /** Sets the customer's fax number (max 32 chars). */
    setFax(number: string): void

    /** Sets the customer's first name. */
    setFirstName(aValue: string): void

    /** Sets the customer's gender. */
    setGender(aValue: number): void

    /** Sets the customer's job title. */
    setJobTitle(aValue: string): void

    /** Sets the customer's last name. */
    setLastName(aValue: string): void

    /** Sets the customer's business phone (max 32 chars). */
    setPhoneBusiness(number: string): void

    /** Sets the customer's home phone (max 32 chars). */
    setPhoneHome(number: string): void

    /** Sets the customer's mobile phone (max 32 chars). */
    setPhoneMobile(number: string): void

    /** Sets the customer's preferred locale. */
    setPreferredLocale(aValue: string): void

    /** Deprecated: use setSalutation(String). */
    setSaluation(salutation: string): void

    /** Sets the customer's salutation. */
    setSalutation(salutation: string): void

    /** Sets the customer's second name. */
    setSecondName(aValue: string): void

    /** Sets the customer's suffix. */
    setSuffix(aValue: string): void

    /** Sets the tax ID (write only when context allows). */
    setTaxID(taxID: string): void

    /** Sets the tax ID type. */
    setTaxIDType(taxIdType: string): void

    /** Sets the customer's title. */
    setTitle(aValue: string): void
}
```
