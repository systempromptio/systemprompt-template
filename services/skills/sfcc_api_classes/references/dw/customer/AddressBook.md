 # dw.customer.AddressBook

 ## Overview
 Represents a customer's collection of addresses (sourced from the Profile object).

 ## Description
 Provides access to a sorted list of `CustomerAddress` entries, the preferred address, and helpers to create, remove, and set the preferred address. The ID of the AddressBook corresponds to the customer's profile ID; data persists on the Profile.

 ```ts
 declare class AddressBook  {
     /** Sorted list of addresses (preferred first) */
     addresses: List<CustomerAddress>

     /** The customer's preferred address */
     preferredAddress: CustomerAddress | null

     getAddresses(): List<CustomerAddress>
     getPreferredAddress(): CustomerAddress | null
     getAddress(id: string): CustomerAddress | null
     createAddress(name: string): CustomerAddress | null
     removeAddress(address: CustomerAddress): void
     setPreferredAddress(anAddress: CustomerAddress | null): void
 }
 ```
